package org.bitcoindevkit.bdkjni

import android.util.Log
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.*
import kotlinx.coroutines.runBlocking
import org.junit.After

import org.junit.Test
import org.junit.runner.RunWith

import org.junit.Assert.*
import org.junit.Before

import org.bitcoindevkit.bdkjni.Types.Network
import org.bitcoindevkit.bdkjni.Types.WalletConstructor
import org.bitcoindevkit.bdkjni.Types.WalletPtr
import org.junit.Ignore

/**
 * Instrumented test, which will execute on an Android device.
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */
@RunWith(AndroidJUnit4::class)
class ExampleInstrumentedTest {

    companion object {
        init {
            System.loadLibrary("bdk_jni")
        }
    }

    private lateinit var wallet: WalletPtr

    @Test
    fun useAppContext() {
        // Context of the app under test.
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        assertEquals("org.bitcoindevkit.bdkjni.test", appContext.packageName)
    }

    @Before
    fun constructor() {
        val dir = createTempDir()
        val descriptor = "wpkh(tprv8ZgxMBicQKsPexGYyaFwnAsCXCjmz2FaTm6LtesyyihjbQE3gRMfXqQBXKM43DvC1UgRVv1qom1qFxNMSqVAs88qx9PhgFnfGVUdiiDf6j4/0/*)"
        val electrum = "tcp://electrum.blockstream.info:60001"
        wallet = Lib().constructor(WalletConstructor("testnet", Network.regtest, dir.toString(), descriptor, null, electrum, null))
        Lib().sync(wallet)
    }

    @Test
    fun newAddress() {
        val address =  Lib().get_new_address(wallet)
        assertFalse(address.isEmpty())
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
                Log.e("BAL_FLOW", "failed flow $id with exception: $e")
                fail()
            }
            .onEach {
                //Log.d("BAL_FLOW", "verifying flow $id, iteration ${it.first}")
                assertFalse(it.second == 0L)
                //Log.d("BAL_FLOW", "finished flow $id iteration ${it.first}")
            }
    }

    @Test
    fun balance() {
        val balance =  Lib().get_balance(wallet)
        assertFalse(balance == 0L)
    }

    @Test
    fun unspent() {
        val unspent =  Lib().list_unspent(wallet)
        assertFalse(unspent.isEmpty())
    }

    @Test
    fun transactions() {
        val transactions =  Lib().list_transactions(wallet)
        assertFalse(transactions.isEmpty())
    }

    @After
    fun destructor() {
        Lib().destructor(wallet)
    }
}
