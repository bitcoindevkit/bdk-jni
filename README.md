# magical-bitcoin-wallet-jni

Build android library in `.aar` format with:
```
./gradlew build
```
Gradle will build automatically the native library with rust for all 4 platforms using NDK.
The output aar library is available at `./build/outputs/aar`.

#### Build native library

Build the library with:

```
CC="aarch64-linux-android21-clang" cargo build --target=aarch64-linux-android
```

Make sure that the compiler from the NDK is in your PATH

If the library is built in debug mode, there should already be a symlink from ./target/debug/<target>/libmagical\_bitcoin\_wallet\_jni.so to the `jniLibs` directory, otherwise manually copy the shared object.
