package org.bitcoindevkit.bdkjni

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.runBlocking

import org.junit.After
import org.junit.Assert.*
import org.junit.Before
import org.junit.Ignore
import org.junit.Test
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import java.io.File

/**
 * Library test, which will execute on linux host.
 *
 */
abstract class LibTest {

    companion object {
        init {
            System.loadLibrary("bdk_jni")
        }
    }

    private val log: Logger = LoggerFactory.getLogger(LibTest::class.java)

    private lateinit var wallet: WalletPtr

    abstract fun getTestDataDir(): String

    fun cleanupTestDataDir() {
        File(getTestDataDir()).deleteRecursively()
    }

    @Before
    fun constructor() {
        val dir = File(getTestDataDir())
        val descriptor =
            "wpkh(tprv8ZgxMBicQKsPexGYyaFwnAsCXCjmz2FaTm6LtesyyihjbQE3gRMfXqQBXKM43DvC1UgRVv1qom1qFxNMSqVAs88qx9PhgFnfGVUdiiDf6j4/0/*)"
        val electrum = "tcp://electrum.blockstream.info:60001"
        wallet = Lib().constructor(
            WalletConstructor(
                "testnet",
                Network.regtest,
                dir.toString(),
                descriptor,
                null,
                electrum,
                null
            )
        )
        Lib().sync(wallet)
    }

    @Test
    fun newAddress() {
        val address = Lib().get_new_address(wallet)
        assertFalse(address.isEmpty())
        log.debug("newAddress: $address")
    }

    @Test
    fun sync() {
        Lib().sync(wallet, 100)
        val balance = Lib().get_balance(wallet)
        assertFalse(balance == 0L)
    }

    // TODO need to figure out why this passes when testing with a localhost node but fails when using blockstream.info
    @Ignore
    @Test
    fun multiThreadBalance() {
        runBlocking {
            val flow1 = newBalanceFlow(1).flowOn(Dispatchers.IO)
            val flow2 = newBalanceFlow(2).flowOn(Dispatchers.IO)
            flow1.flatMapMerge(concurrency = 2) { flow2 }.collect()
            //flow1.collect()
        }
    }

    private fun newBalanceFlow(id: Int): Flow<Pair<Int, Long>> {
        return (1..10).asFlow()
            //.onStart { Log.d("BAL_FLOW", "start flow $id") }
            //.onCompletion { Log.d("BAL_FLOW", "complete flow $id") }
            //.onEach { Log.d("BAL_FLOW", "flow $id, iteration $it") }
            .map {
                val balance = Lib().get_balance(wallet)
                Pair(it, balance)
            }
            .catch { e ->
                log.error("BAL_FLOW", "failed flow $id with exception: $e")
                fail()
            }
            .onEach {
                // log.debug("BAL_FLOW", "verifying flow $id, iteration ${it.first}")
                assertFalse(it.second == 0L)
                // log.debug("BAL_FLOW", "finished flow $id iteration ${it.first}")
            }
    }

    @Test
    fun balance() {
        val balance = Lib().get_balance(wallet)
        assertFalse(balance == 0L)
    }

    @Test
    fun unspent() {
        val unspent = Lib().list_unspent(wallet)
        assertFalse(unspent.isEmpty())
    }

    @Test
    fun transactions() {
        val transactions = Lib().list_transactions(wallet)
        assertFalse(transactions.isEmpty())
    }

    @Test
    fun generate_key() {
        val keys = Lib().generate_extended_key(Network.testnet, 24, "test123")
        assertNotNull(keys)
        assertEquals(24, keys.mnemonic.split(' ').size)
        assertEquals("tprv", keys.xprv.substring(0, 4))
    }

    @Test
    fun restore_key() {
        val mnemonic =
            "shell bid diary primary focus average truly secret lonely circle radar fall tank action place body wedding sponsor embody glue swing gauge shop penalty"
        val keys = Lib().restore_extended_key(Network.testnet, mnemonic, null)
        assertNotNull(keys)
        assertEquals(mnemonic, keys.mnemonic)
        assertEquals(
            "tprv8ZgxMBicQKsPeh5nd4nCDLGh9dLfhqGfUoiQsbThkttjX9oroRY2j5vpEGwkiKiKtzdU7u4eqH2yFicGvz19rMVVXfY8XB9fdoeXWJ7SgVE",
            keys.xprv
        )
    }

    @Test
    fun restore_key_password() {
        val mnemonic =
            "shell bid diary primary focus average truly secret lonely circle radar fall tank action place body wedding sponsor embody glue swing gauge shop penalty"
        val keys = Lib().restore_extended_key(Network.testnet, mnemonic, "test123")
        assertNotNull(keys)
        assertEquals(mnemonic, keys.mnemonic)
        assertEquals(
            "tprv8ZgxMBicQKsPebcVXyErMuuv2rgE34m2SLMBhy4hURbSEAWQ1VsWVVmMnD7FKiAuRrxzAETFnUaSvFNQ5SAS5tYEwsM1KHDpUhLLQgd6yG1",
            keys.xprv
        )
    }

    @After
    fun destructor() {
        Lib().destructor(wallet)
    }
}