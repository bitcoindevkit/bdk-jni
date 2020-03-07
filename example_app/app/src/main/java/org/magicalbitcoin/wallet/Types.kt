package org.magicalbitcoin.wallet.Types

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

    var electrum_url: String,
    var electrum_validate_domain: Boolean?,
    var electrum_proxy: String?
)

data class TxOut(
    var script_pubkey: String,
    // FIXME: should be ULong
    var value: Long
)

data class UTXO(
    var outpoint: String,
    var txout: TxOut
)

// FIXME: Those should be decleared as UBytes, but jackson doesn't know how to parse them. so we use Ints that are larger and won't overflow to negative
data class WalletPtr(
    var raw: List<Int>,
    var id: List<Int>
)