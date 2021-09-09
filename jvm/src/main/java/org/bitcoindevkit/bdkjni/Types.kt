package org.bitcoindevkit.bdkjni

import com.fasterxml.jackson.databind.JsonNode

enum class Network {
    regtest,
    testnet,
}

data class WalletConstructor(
    var name: String,
    var network: Network,
    var path: String,
    var descriptor: String,
    var change_descriptor: String?,

    /**
     * URL of the Electrum server (such as ElectrumX, Esplora, BWT) may start with `ssl://` or `tcp://` and include a port
     * eg. `ssl://electrum.blockstream.info:60002`
     */
    var electrum_url: String,
    /** URL of the socks5 proxy server or a Tor service, null means no proxy */
    var electrum_proxy: String?,
    /** Request retry count */
    var electrum_retry: Int,
    /** Request timeout (seconds), null means no timeout */
    var electrum_timeout: Int?,
    /** Stop searching addresses for transactions after finding an unused gap of this length */
    var electrum_stop_gap: Long,
)

data class TxOut(
    var script_pubkey: String,
    // FIXME: should be ULong
    var value: Long
)

data class UTXO(
    var outpoint: String,
    var txout: TxOut,
    var keychain: String,
)

// FIXME: all Longs should be unsigned
data class TransactionDetails(
    val transaction: JsonNode?,
    val txid: String,
    val received: Long,
    val sent: Long,
    val fee: Long?,
    val confirmation_time: ConfirmationTime?,
    val verified: Boolean
)

data class ConfirmationTime(
    val height: Long,
    val timestamp: Long,
)

data class CreateTxResponse(
    val details: TransactionDetails,
    val psbt: String
)

data class SignResponse(
    val psbt: String,
    val finalized: Boolean
)

data class RawTransaction(
    val transaction: String
)

data class Txid(
    val txid: String
)

data class PublicDescriptorsResponse(
    val external: String,
    val internal: String?
)

data class ExtendedKey(
    val mnemonic: String,
    val xprv: String,
    val fingerprint: String,
)

// FIXME: Those should be decleared as UBytes, but jackson doesn't know how to parse them. so we use Ints that are larger and won't overflow to negative
data class WalletPtr(
    var raw: List<Int>,
    var id: List<Int>
)
