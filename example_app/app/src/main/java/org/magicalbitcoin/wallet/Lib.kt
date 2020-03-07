package org.magicalbitcoin.wallet

import org.magicalbitcoin.wallet.Types.*
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

    fun sync(wallet: WalletPtr, max_address: Int?=null, batch_query_size: Int?=null) {
        val node = JsonNodeFactory.instance.objectNode()
        node.put("wallet", mapper.valueToTree<JsonNode>(wallet))
        node.put("max_address", max_address)
        node.put("batch_query_size", batch_query_size)
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

    fun list_transactions(wallet: WalletPtr, include_raw: Boolean?=false): List<UTXO> {
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
}

