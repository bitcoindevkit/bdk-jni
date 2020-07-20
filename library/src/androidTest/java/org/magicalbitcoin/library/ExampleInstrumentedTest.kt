package org.magicalbitcoin.library

import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.After

import org.junit.Test
import org.junit.runner.RunWith

import org.junit.Assert.*
import org.junit.Before
import org.magicalbitcoin.wallet.Lib
import org.magicalbitcoin.wallet.Types.Network
import org.magicalbitcoin.wallet.Types.WalletConstructor
import org.magicalbitcoin.wallet.Types.WalletPtr

/**
 * Instrumented test, which will execute on an Android device.
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */
@RunWith(AndroidJUnit4::class)
class ExampleInstrumentedTest {

    companion object {
        init {
            System.loadLibrary("magical_bitcoin_wallet_jni")
        }
    }

    private lateinit var wallet: WalletPtr

    @Test
    fun useAppContext() {
        // Context of the app under test.
        val appContext = InstrumentationRegistry.getInstrumentation().targetContext
        assertEquals("org.magicalbitcoin.library.test", appContext.packageName)
    }

    @Before
    fun constructor() {
        val dir = createTempDir()
        val descriptor = "wpkh(tprv8ZgxMBicQKsPexGYyaFwnAsCXCjmz2FaTm6LtesyyihjbQE3gRMfXqQBXKM43DvC1UgRVv1qom1qFxNMSqVAs88qx9PhgFnfGVUdiiDf6j4/0/*)"
        val electrum = "tcp://testnet.aranguren.org:51001"
        wallet = Lib().constructor(WalletConstructor("testnet", Network.regtest, dir.toString(), descriptor, null, electrum, null))
        Lib().sync(wallet)
    }

    @Test
    fun newAddress() {
        val address =  Lib().get_new_address(wallet)
        assertFalse(address.isEmpty())
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
    fun desctructor() {
        Lib().destructor(wallet)
    }
}
