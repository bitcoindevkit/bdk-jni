use std::any::TypeId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use log::{debug, error, info, trace};

use magical_bitcoin_wallet::bitcoin;
use magical_bitcoin_wallet::electrum_client::client::{
    ElectrumPlaintextStream, ElectrumProxyStream, ElectrumSslStream,
};
use magical_bitcoin_wallet::sled;
use magical_bitcoin_wallet::{Client, Wallet};

use bitcoin::Network;

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "method", content = "params")]
#[serde(rename_all = "snake_case")]
enum MagicalRequest {
    Constructor {
        name: String,
        network: Network,
        path: PathBuf,
        descriptor: String,
        change_descriptor: Option<String>,

        // <nothing> = tcp:// | tcp:// | ssl://
        electrum_url: String,
        electrum_validate_domain: Option<bool>,
        // <nothing> | socks5://
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
        batch_query_size: Option<usize>,
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
}

#[derive(Debug)]
enum MagicalError {
    WalletError(magical_bitcoin_wallet::error::Error),
    ElectrumClientError(magical_bitcoin_wallet::electrum_client::types::Error),
    Serialization(serde_json::error::Error),

    Unsupported(String),
    CantOpenDb(sled::Error, PathBuf),
    CantOpenTree(sled::Error, String),
}

impl From<magical_bitcoin_wallet::error::Error> for MagicalError {
    fn from(other: magical_bitcoin_wallet::error::Error) -> Self {
        MagicalError::WalletError(other)
    }
}

impl From<magical_bitcoin_wallet::electrum_client::types::Error> for MagicalError {
    fn from(other: magical_bitcoin_wallet::electrum_client::types::Error) -> Self {
        MagicalError::ElectrumClientError(other)
    }
}

#[derive(Debug, Clone)]
struct OpaquePtr<T> {
    raw: *const T,
    id: TypeId,
}

impl<T: 'static> OpaquePtr<T> {
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

    fn move_out(self) -> Box<T> {
        unsafe { Box::from_raw(self.raw as *mut T) }
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

fn do_constructor_call(req: MagicalRequest) -> Result<serde_json::Value, MagicalError> {
    use crate::MagicalRequest::*;

    if let Constructor {
        name,
        network,
        path,
        descriptor,
        change_descriptor,
        electrum_url,
        electrum_validate_domain,
        electrum_proxy,
    } = req
    {
        let database =
            sled::open(path.clone()).map_err(|e| MagicalError::CantOpenDb(e, path.clone()))?;
        let tree = database
            .open_tree(name.clone())
            .map_err(|e| MagicalError::CantOpenTree(e, name.clone()))?;

        debug!(
            "Database at {} name {} opened successfully",
            path.as_path().display(),
            name
        );

        if let Some(proxy) = electrum_proxy {
            let proxy = proxy.replacen("socks5://", "", 1);
            let electrum_url = if electrum_url.starts_with("ssl://") {
                return Err(MagicalError::Unsupported(
                    "SSL is not supported over a proxy".to_string(),
                ));
            } else {
                electrum_url.replacen("tcp://", "", 1)
            };

            let client = Client::new_proxy(electrum_url.as_str(), proxy)?;
            let ptr: OpaquePtr<_> = Wallet::new(
                &descriptor,
                change_descriptor.as_deref(),
                network,
                tree,
                client,
            )?
            .into();
            serde_json::to_value(&ptr).map_err(MagicalError::Serialization)
        } else if electrum_url.starts_with("ssl://") {
            let electrum_url = electrum_url.replacen("ssl://", "", 1);
            let client = Client::new_ssl(
                electrum_url.as_str(),
                electrum_validate_domain.unwrap_or(false),
            )?;
            let ptr: OpaquePtr<_> = Wallet::new(
                &descriptor,
                change_descriptor.as_deref(),
                network,
                tree,
                client,
            )?
            .into();
            serde_json::to_value(&ptr).map_err(MagicalError::Serialization)
        } else {
            let electrum_url = electrum_url.replacen("tcp://", "", 1);
            let client = Client::new(electrum_url)?;
            let ptr: OpaquePtr<_> = Wallet::new(
                &descriptor,
                change_descriptor.as_deref(),
                network,
                tree,
                client,
            )?
            .into();
            serde_json::to_value(&ptr).map_err(MagicalError::Serialization)
        }
    } else {
        Err(MagicalError::Unsupported(
            "Called `do_constructor_call` with a non-Constructor request".to_string(),
        ))
    }
}

fn do_wallet_call<S, D>(
    wallet: Box<Wallet<S, D>>,
    req: MagicalRequest,
) -> Result<serde_json::Value, MagicalError>
where
    S: std::io::Read + std::io::Write,
    D: magical_bitcoin_wallet::database::BatchDatabase,
{
    use crate::MagicalRequest::*;

    let resp = match req {
        Constructor { .. } => {
            return Err(MagicalError::Unsupported(
                "Called `do_wallet_call` with a Constructor request".to_string(),
            ))
        }
        Destructor { .. } => Ok(serde_json::Value::Null),
        GetNewAddress { .. } => {
            serde_json::to_value(&wallet.get_new_address()?).map_err(MagicalError::Serialization)
        }
        Sync {
            max_address,
            batch_query_size,
            ..
        } => serde_json::to_value(&wallet.sync(max_address, batch_query_size)?)
            .map_err(MagicalError::Serialization),
        ListUnspent { .. } => {
            serde_json::to_value(&wallet.list_unspent()?).map_err(MagicalError::Serialization)
        }
        GetBalance { .. } => {
            serde_json::to_value(&wallet.get_balance()?).map_err(MagicalError::Serialization)
        }
        ListTransactions { include_raw, .. } => {
            serde_json::to_value(&wallet.list_transactions(include_raw.unwrap_or(false))?)
                .map_err(MagicalError::Serialization)
        }
    };

    if let Destructor { .. } = req {
        std::mem::drop(wallet);
    } else {
        std::mem::forget(wallet);
    }

    resp
}

/// Expose the JNI interface below
#[cfg(target_os = "android")]
#[allow(non_snake_case)]
pub mod android {
    use std::ffi::{CStr, CString};

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

    #[no_mangle]
    pub unsafe extern "C" fn Java_org_magicalbitcoin_wallet_Lib_call(
        env: JNIEnv,
        _: JClass,
        incoming_string: JString,
    ) -> jstring {
        use crate::MagicalRequest::*;

        android_logger::init_once(
            android_logger::Config::default().with_min_level(log::Level::Debug),
        );

        let incoming_cstr = match env.get_string(incoming_string) {
            Ok(string) => CStr::from_ptr(string.as_ptr()),
            Err(e) => {
                return JNIError {
                    error: format!("Invalid input string: {:?}", e),
                    code: -1001,
                }
                .into_string(&env)
            }
        };

        let incoming_str = match incoming_cstr.to_str() {
            Ok(string) => string,
            Err(e) => {
                return JNIError {
                    error: format!("Invalid input string encoding: {:?}", e),
                    code: -1002,
                }
                .into_string(&env)
            }
        };

        let deser = match serde_json::from_str::<MagicalRequest>(incoming_str) {
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
            | ListTransactions { ref wallet, .. } => {
                if let Ok(w) =
                    OpaquePtr::<Wallet<ElectrumPlaintextStream, sled::Tree>>::convert_from(wallet)
                {
                    do_wallet_call(w.move_out(), deser)
                } else if let Ok(w) =
                    OpaquePtr::<Wallet<ElectrumSslStream, sled::Tree>>::convert_from(wallet)
                {
                    do_wallet_call(w.move_out(), deser)
                } else if let Ok(w) =
                    OpaquePtr::<Wallet<ElectrumProxyStream, sled::Tree>>::convert_from(wallet)
                {
                    do_wallet_call(w.move_out(), deser)
                } else {
                    Err(MagicalError::Unsupported("Unknown Wallet type".to_string()))
                }
            }
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
