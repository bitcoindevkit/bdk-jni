package com.afilini.myapplication

import android.os.Bundle
import android.util.Log
import androidx.appcompat.app.AppCompatActivity
import android.view.Menu
import android.view.MenuItem

import kotlinx.android.synthetic.main.activity_main.*
import org.magicalbitcoin.wallet.Lib
import org.magicalbitcoin.wallet.Types.Network
import org.magicalbitcoin.wallet.Types.WalletConstructor

class MainActivity : AppCompatActivity() {
    companion object {
        init {
            System.loadLibrary("magical_bitcoin_wallet_jni")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        setSupportActionBar(toolbar)

        fab.setOnClickListener { view ->
            val lib = Lib()
            val result = lib.construct_wallet(WalletConstructor("test", Network.regtest, filesDir.toString(), "wpkh(tprv8ZgxMBicQKsPexGYyaFwnAsCXCjmz2FaTm6LtesyyihjbQE3gRMfXqQBXKM43DvC1UgRVv1qom1qFxNMSqVAs88qx9PhgFnfGVUdiiDf6j4/0/*)", null, "tcp://tn.not.fyi:55001", null, null))
            Log.i("result", result.toString())
            val new_addr = lib.get_new_address(result)
            Log.i("new_address", new_addr)
        }
    }

    override fun onCreateOptionsMenu(menu: Menu): Boolean {
        // Inflate the menu; this adds items to the action bar if it is present.
        menuInflater.inflate(R.menu.menu_main, menu)
        return true
    }

    override fun onOptionsItemSelected(item: MenuItem): Boolean {
        // Handle action bar item clicks here. The action bar will
        // automatically handle clicks on the Home/Up button, so long
        // as you specify a parent activity in AndroidManifest.xml.
        return when (item.itemId) {
            R.id.action_settings -> true
            else -> super.onOptionsItemSelected(item)
        }
    }
}
