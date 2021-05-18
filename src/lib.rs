#[macro_use]
extern crate serde_json;
extern crate rand;

use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::str::FromStr;

use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use log::{debug, error, info, trace};

use bdk::electrum_client;
use bdk::sled;
use bdk::Wallet;
use bdk::{bitcoin, KeychainKind};

use bdk::bitcoin::secp256k1::Secp256k1;
use bdk::blockchain::{noop_progress, ElectrumBlockchain};
use bdk::{FeeRate, TransactionDetails};

use electrum_client::Client;

use bdk::keys::bip39::{Language, Mnemonic, MnemonicType};
use bdk::keys::{DerivableKey, ExtendedKey, GeneratableKey, GeneratedKey};
use bdk::miniscript::miniscript;
use bdk::wallet::AddressIndex::New;
use bitcoin::consensus::encode::{deserialize, serialize};
use bitcoin::hashes::hex::{FromHex, ToHex};
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoin::{Address, Network, OutPoint, Transaction};

#[derive(Debug, Deserialize)]
struct KotlinPair<F: std::fmt::Debug, S: std::fmt::Debug> {
    #[serde(bound(serialize = "F: Deserialize<'de>"))]
    first: F,
    #[serde(bound(serialize = "F: Deserialize<'de>"))]
    second: S,
}

impl<F: std::fmt::Debug, S: std::fmt::Debug> From<KotlinPair<F, S>> for (F, S) {
    fn from(other: KotlinPair<F, S>) -> Self {
        (other.first, other.second)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "snake_case")]
enum BDKRequest {
    Constructor {
        name: String,
        network: Network,
        path: PathBuf,
        descriptor: String,
        change_descriptor: Option<String>,

        electrum_url: String,
        electrum_proxy: Option<String>,
    },
    Destructor {
        wallet: IntermediatePtr,
    },
    GetNewAddress {
        wallet: IntermediatePtr,
    },
    Sync {
        wallet: IntermediatePtr,

        max_address: Option<u32>,
    },
    ListUnspent {
        wallet: IntermediatePtr,
    },
    GetBalance {
        wallet: IntermediatePtr,
    },
    ListTransactions {
        wallet: IntermediatePtr,

        include_raw: Option<bool>,
    },
    CreateTx {
        wallet: IntermediatePtr,

        fee_rate: f32,
        // store the amounts as strings to avoid rounding errors
        addressees: Vec<KotlinPair<String, String>>,

        unspendable: Option<Vec<String>>,
        utxos: Option<Vec<String>>,
        send_all: Option<bool>,
        policy: Option<BTreeMap<String, Vec<usize>>>,
    },
    Sign {
        wallet: IntermediatePtr,

        psbt: String,

        assume_height: Option<u32>,
    },
    ExtractPsbt {
        wallet: IntermediatePtr,

        psbt: String,
    },
    Broadcast {
        wallet: IntermediatePtr,

        raw_tx: String,
    },
    PublicDescriptors {
        wallet: IntermediatePtr,
    },
    /// Generate new random seed mnemonic phrase and corresponding master extended key
    GenerateExtendedKey {
        network: Network,
        word_count: usize,
        password: Option<String>,
    },
    /// Restore a master extended key from seed backup mnemonic words
    RestoreExtendedKey {
        network: Network,
        mnemonic: String,
        password: Option<String>,
    },
}

#[derive(Debug)]
enum BDKJNIError {
    WalletError(bdk::Error),
    ElectrumClientError(bdk::electrum_client::Error),
    Serialization(serde_json::error::Error),

    Unsupported(String),
    CantOpenDb(sled::Error, PathBuf),
    CantOpenTree(sled::Error, String),

    Parsing(String),

    ExtKeyError(bdk::keys::KeyError),
}

impl From<bdk::Error> for BDKJNIError {
    fn from(other: bdk::Error) -> Self {
        match other {
            bdk::Error::Electrum(e) => BDKJNIError::ElectrumClientError(e),
            e => BDKJNIError::WalletError(e),
        }
    }
}

impl From<bdk::electrum_client::Error> for BDKJNIError {
    fn from(other: bdk::electrum_client::Error) -> Self {
        BDKJNIError::ElectrumClientError(other)
    }
}

impl From<bdk::keys::KeyError> for BDKJNIError {
    fn from(other: bdk::keys::KeyError) -> Self {
        BDKJNIError::ExtKeyError(other)
    }
}

#[derive(Debug, Clone)]
struct OpaquePtr<T> {
    raw: *const T,
    id: TypeId,
}

impl<T: 'static> OpaquePtr<T> {
    #[allow(dead_code)]
    fn convert_from(value: &IntermediatePtr) -> Result<OpaquePtr<T>, ()> {
        let mut hasher = DefaultHasher::new();
        TypeId::of::<T>().hash(&mut hasher);

        if hasher.finish().to_be_bytes() == value.id {
            Ok(OpaquePtr {
                raw: u64::from_be_bytes(value.raw) as *const T,
                id: TypeId::of::<T>(),
            })
        } else {
            Err(())
        }
    }

    #[allow(dead_code)]
    fn move_out(self) -> Box<T> {
        unsafe { Box::from_raw(self.raw as *mut T) }
    }

    #[allow(dead_code)]
    fn as_ref(&self) -> &T {
        unsafe { &*self.raw }
    }
}

impl<T> Serialize for OpaquePtr<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("OpaquePtr", 2)?;
        state.serialize_field("raw", &(self.raw as u64).to_be_bytes())?;

        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        state.serialize_field("id", &hasher.finish().to_be_bytes())?;
        state.end()
    }
}

impl<T: 'static> From<T> for OpaquePtr<T> {
    fn from(other: T) -> Self {
        OpaquePtr {
            id: TypeId::of::<T>(),
            raw: Box::into_raw(Box::new(other)),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct IntermediatePtr {
    raw: [u8; 8],
    id: [u8; 8],
}

#[allow(dead_code)]
fn do_constructor_call(req: BDKRequest) -> Result<serde_json::Value, BDKJNIError> {
    use crate::BDKRequest::*;

    if let Constructor {
        name,
        network,
        path,
        descriptor,
        change_descriptor,
        electrum_url,
        electrum_proxy: _electrum_proxy, // TODO enable use of proxy
    } = req
    {
        let database =
            sled::open(path.clone()).map_err(|e| BDKJNIError::CantOpenDb(e, path.clone()))?;
        let tree = database
            .open_tree(name.clone())
            .map_err(|e| BDKJNIError::CantOpenTree(e, name.clone()))?;

        debug!(
            "Database at {} name {} opened successfully",
            path.as_path().display(),
            name
        );

        let descriptor: &str = descriptor.as_str();
        let change_descriptor: Option<&str> = change_descriptor.as_deref();

        let client = Client::new(&electrum_url)?;
        let ptr: OpaquePtr<_> = Wallet::new(
            descriptor,
            change_descriptor,
            network,
            tree,
            ElectrumBlockchain::from(client),
        )?
        .into();

        serde_json::to_value(&ptr).map_err(BDKJNIError::Serialization)
    } else {
        Err(BDKJNIError::Unsupported(
            "Called `do_constructor_call` with a non-Constructor request".to_string(),
        ))
    }
}

#[allow(dead_code)]
fn do_wallet_call<S, D>(
    wallet: &Wallet<S, D>,
    req: BDKRequest,
) -> Result<serde_json::Value, BDKJNIError>
where
    S: bdk::blockchain::Blockchain,
    D: bdk::database::BatchDatabase,
{
    use crate::BDKRequest::*;

    let resp = match req {
        Constructor { .. } => {
            return Err(BDKJNIError::Unsupported(
                "Called `do_wallet_call` with a Constructor request".to_string(),
            ))
        }
        Destructor { .. } => Ok(serde_json::Value::Null),
        GetNewAddress { .. } => {
            serde_json::to_value(&wallet.get_address(New)?).map_err(BDKJNIError::Serialization)
        }
        Sync { max_address, .. } => {
            serde_json::to_value(&wallet.sync(noop_progress(), max_address)?)
                .map_err(BDKJNIError::Serialization)
        }
        ListUnspent { .. } => {
            serde_json::to_value(&wallet.list_unspent()?).map_err(BDKJNIError::Serialization)
        }
        GetBalance { .. } => {
            serde_json::to_value(&wallet.get_balance()?).map_err(BDKJNIError::Serialization)
        }
        ListTransactions { include_raw, .. } => {
            serde_json::to_value(&wallet.list_transactions(include_raw.unwrap_or(false))?)
                .map_err(BDKJNIError::Serialization)
        }
        CreateTx {
            fee_rate,
            unspendable,
            utxos,
            addressees,
            send_all,
            policy,
            ..
        } => {
            #[derive(Serialize)]
            struct CreateTxResponse {
                details: TransactionDetails,
                psbt: String,
            }

            let addressees = addressees
                .into_iter()
                .map(|pair| -> Result<_, Box<dyn std::error::Error>> {
                    let (a, v) = pair.into();
                    Ok((Address::from_str(&a)?.script_pubkey(), v.parse()?))
                })
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| BDKJNIError::Parsing(format!("{:?}", e)))?;

            let mut builder = wallet.build_tx();
            builder.fee_rate(FeeRate::from_sat_per_vb(fee_rate));

            if send_all == Some(true) {
                builder
                    .drain_wallet()
                    .set_single_recipient(addressees[0].0.clone());
            } else {
                builder.set_recipients(addressees);
            }

            let utxos: Option<Vec<OutPoint>> = utxos
                .map(|u| {
                    u.into_iter()
                        .map(|s| s.parse())
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()
                .map_err(|e| BDKJNIError::Parsing(format!("{:?}", e)))?;
            let unspendable: Option<Vec<OutPoint>> = unspendable
                .map(|u| {
                    u.into_iter()
                        .map(|s| s.parse())
                        .collect::<Result<Vec<_>, _>>()
                })
                .transpose()
                .map_err(|e| BDKJNIError::Parsing(format!("{:?}", e)))?;

            if let Some(utxos) = utxos {
                builder.add_utxos(utxos.as_slice())?;
            }
            if let Some(unspendable) = unspendable {
                builder.unspendable(unspendable);
            }

            if let Some(policy_path) = policy {
                builder.policy_path(policy_path, KeychainKind::External);
            }

            let (psbt, details) = builder.finish()?;
            serde_json::to_value(&CreateTxResponse {
                details,
                psbt: base64::encode(&serialize(&psbt)),
            })
            .map_err(BDKJNIError::Serialization)
        }
        Sign {
            psbt,
            assume_height,
            ..
        } => {
            #[derive(Serialize)]
            struct SignResponse {
                psbt: String,
                finalized: bool,
            }

            let psbt =
                base64::decode(&psbt).map_err(|e| BDKJNIError::Parsing(format!("{:?}", e)))?;
            let psbt = deserialize(&psbt).map_err(|e| BDKJNIError::Parsing(format!("{:?}", e)))?;

            let (psbt, finalized) = wallet.sign(psbt, assume_height)?;

            serde_json::to_value(&SignResponse {
                psbt: base64::encode(&serialize(&psbt)),
                finalized,
            })
            .map_err(BDKJNIError::Serialization)
        }
        ExtractPsbt { psbt, .. } => {
            let psbt =
                base64::decode(&psbt).map_err(|e| BDKJNIError::Parsing(format!("{:?}", e)))?;
            let psbt: PartiallySignedTransaction =
                deserialize(&psbt).map_err(|e| BDKJNIError::Parsing(format!("{:?}", e)))?;

            Ok(json!({
                "transaction": serialize(&psbt.extract_tx()).to_hex(),
            }))
        }
        Broadcast { raw_tx, .. } => {
            let raw_tx: Vec<u8> =
                FromHex::from_hex(&raw_tx).map_err(|e| BDKJNIError::Parsing(format!("{:?}", e)))?;
            let raw_tx: Transaction =
                deserialize(&raw_tx).map_err(|e| BDKJNIError::Parsing(format!("{:?}", e)))?;

            let txid = wallet.broadcast(raw_tx)?;

            Ok(json!({
                "txid": txid.to_hex(),
            }))
        }
        PublicDescriptors { .. } => {
            #[derive(Serialize)]
            struct PublicDescriptorsResponse {
                external: String,
                internal: Option<String>,
            }

            let external = wallet
                .public_descriptor(KeychainKind::External)?
                .unwrap()
                .to_string();
            let internal = wallet
                .public_descriptor(KeychainKind::Internal)?
                .map(|d| d.to_string());

            serde_json::to_value(&PublicDescriptorsResponse { external, internal })
                .map_err(BDKJNIError::Serialization)
        }
        GenerateExtendedKey { .. } => Err(BDKJNIError::Unsupported(
            "Called `do_wallet_call` with a GenerateExtendedKey request".to_string(),
        )),
        RestoreExtendedKey { .. } => Err(BDKJNIError::Unsupported(
            "Called `do_wallet_call` with a RestoreExtendedKey request".to_string(),
        )),
    };

    resp
}

#[allow(dead_code)]
fn do_key_call(req: BDKRequest) -> Result<serde_json::Value, BDKJNIError> {
    use crate::BDKRequest::*;

    let secp = Secp256k1::new();

    match req {
        GenerateExtendedKey {
            network,
            word_count,
            password,
        } => {
            #[derive(Serialize)]
            struct GenerateExtendedKeyResponse {
                mnemonic: String,
                xprv: String,
                fingerprint: String,
            }
            let mnemonic_type = match word_count {
                12 => MnemonicType::Words12,
                _ => MnemonicType::Words24,
            };
            let mnemonic: GeneratedKey<_, miniscript::BareCtx> =
                Mnemonic::generate((mnemonic_type, Language::English)).unwrap();
            let mnemonic = mnemonic.into_key();
            let xkey: ExtendedKey = (mnemonic.clone(), password).into_extended_key()?;
            let xprv = xkey.into_xprv(network).unwrap();
            let fingerprint = xprv.fingerprint(&secp);

            let resp = &GenerateExtendedKeyResponse {
                mnemonic: mnemonic.to_string(),
                xprv: xprv.to_string(),
                fingerprint: fingerprint.to_string(),
            };

            serde_json::to_value(resp).map_err(BDKJNIError::Serialization)
        }
        RestoreExtendedKey {
            network,
            mnemonic,
            password,
        } => {
            #[derive(Serialize)]
            struct RestoreExtendedKeyResponse {
                mnemonic: String,
                xprv: String,
                fingerprint: String,
            }
            let mnemonic = Mnemonic::from_phrase(mnemonic.as_ref(), Language::English).unwrap();
            let xkey: ExtendedKey = (mnemonic.clone(), password).into_extended_key()?;
            let xprv = xkey.into_xprv(network).unwrap();
            let fingerprint = xprv.fingerprint(&secp);
            let resp = RestoreExtendedKeyResponse {
                mnemonic: mnemonic.to_string(),
                xprv: xprv.to_string(),
                fingerprint: fingerprint.to_string(),
            };

            serde_json::to_value(resp).map_err(BDKJNIError::Serialization)
        }
        _ => Err(BDKJNIError::Unsupported(
            "Called `do_key_call` with a non-keys request".to_string(),
        )),
    }
}

/// Expose the JNI interface below
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub mod android {
    use std::ffi::CString;

    use jni::objects::{JClass, JObject, JString};
    use jni::sys::jstring;
    use jni::JNIEnv;

    use crate::*;

    #[derive(Debug, Serialize)]
    struct JNIError {
        error: String,
        code: i32,
    }

    fn string_to_jstring(env: &JNIEnv, input: &str) -> Result<jstring, String> {
        let cstring = CString::new(input).map_err(|e| format!("{:?}", e))?;
        let cstr = cstring.to_str().map_err(|e| format!("{:?}", e))?;

        let output = env.new_string(cstr).map_err(|e| format!("{:?}", e))?;
        Ok(output.into_inner())
    }

    impl JNIError {
        fn into_string(self, env: &JNIEnv) -> jstring {
            let serialized = serde_json::to_string(&self)
                .unwrap_or("{\"error\": \"Can't serialize error\", \"code\": -1000}".to_string());
            string_to_jstring(env, &serialized).unwrap_or(JObject::null().into_inner())
        }
    }

    impl Into<String> for JNIError {
        fn into(self) -> String {
            serde_json::to_string(&self)
                .unwrap_or("{\"error\": \"Can't serialize error\", \"code\": -1000}".to_string())
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn Java_org_bitcoindevkit_bdkjni_Lib_call(
        env: JNIEnv,
        _: JClass,
        incoming_stringj: JString,
    ) -> jstring {
        use crate::BDKRequest::*;

        android_logger::init_once(
            android_logger::Config::default().with_min_level(log::Level::Debug),
        );

        let incoming_string: String = match env.get_string(incoming_stringj) {
            Ok(string) => string.into(),
            Err(e) => {
                return JNIError {
                    error: format!("Invalid input string: {:?}", e),
                    code: -1001,
                }
                .into_string(&env)
            }
        };

        let deser = match serde_json::from_str::<BDKRequest>(incoming_string.as_str()) {
            Ok(req) => req,
            Err(e) => {
                return JNIError {
                    error: format!("Cannot deserialize input: {:?}", e),
                    code: -1003,
                }
                .into_string(&env)
            }
        };

        let response_result = match &deser {
            Constructor { .. } => do_constructor_call(deser),
            Destructor { ref wallet }
            | GetNewAddress { ref wallet }
            | Sync { ref wallet, .. }
            | ListUnspent { ref wallet }
            | GetBalance { ref wallet }
            | ListTransactions { ref wallet, .. }
            | CreateTx { ref wallet, .. }
            | Sign { ref wallet, .. }
            | ExtractPsbt { ref wallet, .. }
            | Broadcast { ref wallet, .. }
            | PublicDescriptors { ref wallet } => {
                if let Ok(w) =
                    OpaquePtr::<Wallet<ElectrumBlockchain, sled::Tree>>::convert_from(wallet)
                {
                    let drop_wallet = matches!(deser, Destructor { .. });
                    let result = do_wallet_call(w.as_ref(), deser);

                    if drop_wallet {
                        let _ = w.move_out();
                    }

                    result
                } else {
                    Err(BDKJNIError::Unsupported(
                        "Invalid wallet pointer".to_string(),
                    ))
                }
            }
            GenerateExtendedKey { .. } | RestoreExtendedKey { .. } => do_key_call(deser),
        };

        let final_string = match response_result {
            Ok(stuff) => serde_json::to_string(&stuff),
            Err(e) => {
                return JNIError {
                    error: format!("{:?}", e),
                    code: -1,
                }
                .into_string(&env)
            }
        };
        let final_string = match final_string {
            Ok(string) => string,
            Err(e) => {
                return JNIError {
                    error: format!("{:?}", e),
                    code: -1004,
                }
                .into_string(&env)
            }
        };

        string_to_jstring(&env, &final_string).unwrap_or(JObject::null().into_inner())
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_opaque_ptr() {
        let string = String::from("HelloWorld!");
        let ptr: OpaquePtr<_> = string.into();

        let val = serde_json::to_value(&ptr).unwrap();
        assert!(val.is_object());
        assert!(val.as_object().unwrap().contains_key("raw"));
        assert!(val.as_object().unwrap().contains_key("id"));

        let deser: IntermediatePtr = serde_json::from_value(val).unwrap();

        assert!(OpaquePtr::<&str>::convert_from(&deser).is_err());
        assert!(OpaquePtr::<String>::convert_from(&deser).is_ok());
    }
}
