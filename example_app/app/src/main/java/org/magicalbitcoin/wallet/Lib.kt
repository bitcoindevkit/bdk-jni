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

    fun construct_wallet(data: WalletConstructor): WalletPtr {
        val req = JsonRpc("constructor", mapper.valueToTree(data))
        val reqString = mapper.writeValueAsString(req)
        val resString = call(reqString)
        val json: JsonNode = mapper.readValue(resString)
        if (json.has("error")) {
            throw Exception(json.get("error").asText())
        }
        return mapper.treeToValue(json, WalletPtr::class.java)
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
}

