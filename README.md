# bdk-jni

Make sure that you have an Android NDK installed (preferibly the latest one), and that you have an `NDK_HOME` env variable set before you start building the library. Usually, if installed through the `sdkmanager`,
your `NDK_HOME` will look more or less like this: `/home/user/Android/Sdk/ndk/<version>/`.

Build android library in `.aar` format with:
```
./gradlew build
```
Gradle will build automatically the native library with rust for all 4 platforms using NDK. You can choose to build only for a specific platform by setting the env variable `BUILD_TARGETS` to a comma-separated list
containing one or more of the following items:

* `aarch64`
* `armv7`
* `x86_64`
* `i686`

The output aar library is available at `./build/outputs/aar`.

You can run the tests with:
```
./gradlew connectedDebugAndroidTest
```

#### Build just the native library

If you only want to build the native library, maybe for one single platform, you can do so with something like:

```
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="aarch64-linux-android21-clang" CC="aarch64-linux-android21-clang" cargo build --target=aarch64-linux-android
```

Make sure that the compiler from the NDK is in your PATH

If the library is built in debug mode, there should already be a symlink from ./target/debug/<target>/libbdk\_jni.so to the `jniLibs` directory, otherwise manually copy the shared object.
