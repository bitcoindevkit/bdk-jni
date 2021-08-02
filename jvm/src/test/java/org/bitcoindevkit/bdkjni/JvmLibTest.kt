package org.bitcoindevkit.bdkjni

import java.nio.file.Files
import java.nio.file.Paths

/**
 * Library test, which will execute on linux host.
 *
 */
class JvmLibTest : LibTest() {

    override fun getDataDir(): String {
        //return Files.createTempDirectory("bdk-test").toString()
        return Paths.get(System.getProperty("java.io.tmpdir"), "bdk-test-${randomDirId()}").toString()
    }

}
