package org.bitcoindevkit.bdkjni

import org.bitcoindevkit.bdkjni.Types.*
import com.fasterxml.jackson.databind.JsonNode
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.databind.node.JsonNodeFactory
import com.fasterxml.jackson.module.kotlin.KotlinModule
import com.fasterxml.jackson.module.kotlin.readValue

class Lib {
    external fun call(pattern: String): String

    data class JsonRpc(var method: String, var params: JsonNode)

    val mapper = ObjectMapper()
        .registerModule(KotlinModule())



    companion object {
        @JvmStatic
        fun load() {
            System.loadLibrary("bdk_jni")
        }
    }

    fun constructor(data: WalletConstructor): WalletPtr {
        val req = JsonRpc("constructor", mapper.valueToTree(data))
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
        return mapper.treeToValue(json, WalletPtr::class.java)
    }

    fun destructor(wallet: WalletPtr) {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("wallet", mapper.valueToTree<JsonNode>(wallet))
        val req = JsonRpc("destructor", node)
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
    }

    fun get_new_address(wallet: WalletPtr): String {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("wallet", mapper.valueToTree<JsonNode>(wallet))
        val req = JsonRpc("get_new_address", node)
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
        return json.asText()
    }

    fun sync(wallet: WalletPtr, max_address: Int?=null) {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("wallet", mapper.valueToTree<JsonNode>(wallet))
        node.put("max_address", max_address)
        val req = JsonRpc("sync", node)
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
    }

    fun list_unspent(wallet: WalletPtr): List<UTXO> {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("wallet", mapper.valueToTree<JsonNode>(wallet))
        val req = JsonRpc("list_unspent", node)
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
        // FIXME: would be better to re-use the jsonnode instead of parsing the string again
        return mapper.readValue(resString, mapper.typeFactory.constructCollectionType(List::class.java, UTXO::class.java))
    }

    // FIXME: should be ULong
    fun get_balance(wallet: WalletPtr): Long {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("wallet", mapper.valueToTree<JsonNode>(wallet))
        val req = JsonRpc("get_balance", node)
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
        return json.asLong()
    }

    fun list_transactions(wallet: WalletPtr, include_raw: Boolean?=false): List<TransactionDetails> {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("wallet", mapper.valueToTree<JsonNode>(wallet))
        node.put("include_raw", mapper.valueToTree<JsonNode>(include_raw))
        val req = JsonRpc("list_transactions", node)
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
        // FIXME: would be better to re-use the jsonnode instead of parsing the string again
        return mapper.readValue(resString, mapper.typeFactory.constructCollectionType(List::class.java, TransactionDetails::class.java))
    }

    fun create_tx(wallet: WalletPtr, fee_rate: Float, addressees: List<Pair<String, String>>, send_all: Boolean?=false, utxos: List<String>?=null, unspendable: List<String>?=null, policy: Map<String, List<String>>?=null): CreateTxResponse {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("wallet", mapper.valueToTree<JsonNode>(wallet))
        node.put("fee_rate", mapper.valueToTree<JsonNode>(fee_rate))
        node.put("addressees", mapper.valueToTree<JsonNode>(addressees))
        node.put("send_all", mapper.valueToTree<JsonNode>(send_all))
        node.put("utxos", mapper.valueToTree<JsonNode>(utxos))
        node.put("unspendable", mapper.valueToTree<JsonNode>(unspendable))
        node.put("policy", mapper.valueToTree<JsonNode>(policy))
        val req = JsonRpc("create_tx", node)
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
        // FIXME: would be better to re-use the jsonnode instead of parsing the string again
        return mapper.treeToValue(json, CreateTxResponse::class.java)
    }

    fun sign(wallet: WalletPtr, psbt: String, assume_height: Int?=null): SignResponse {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("wallet", mapper.valueToTree<JsonNode>(wallet))
        node.put("psbt", mapper.valueToTree<JsonNode>(psbt))
        node.put("assume_height", mapper.valueToTree<JsonNode>(assume_height))
        val req = JsonRpc("sign", node)
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
        // FIXME: would be better to re-use the jsonnode instead of parsing the string again
        return mapper.treeToValue(json, SignResponse::class.java)
    }

    fun extract_psbt(wallet: WalletPtr, psbt: String): RawTransaction {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("wallet", mapper.valueToTree<JsonNode>(wallet))
        node.put("psbt", mapper.valueToTree<JsonNode>(psbt))
        val req = JsonRpc("extract_psbt", node)
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
        // FIXME: would be better to re-use the jsonnode instead of parsing the string again
        return mapper.treeToValue(json, RawTransaction::class.java)
    }

    fun broadcast(wallet: WalletPtr, raw_tx: String): Txid {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("wallet", mapper.valueToTree<JsonNode>(wallet))
        node.put("raw_tx", mapper.valueToTree<JsonNode>(raw_tx))
        val req = JsonRpc("broadcast", node)
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
        // FIXME: would be better to re-use the jsonnode instead of parsing the string again
        return mapper.treeToValue(json, Txid::class.java)
    }

    fun public_descriptors(wallet: WalletPtr): PublicDescriptorsResponse {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("wallet", mapper.valueToTree<JsonNode>(wallet))
        val req = JsonRpc("public_descriptors", node)
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
        // FIXME: would be better to re-use the jsonnode instead of parsing the string again
        return mapper.treeToValue(json, PublicDescriptorsResponse::class.java)
    }
}
