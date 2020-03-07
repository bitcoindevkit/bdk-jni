# magical-bitcoin-wallet-jni

Build the library with:

```
CC="aarch64-linux-android21-clang" cargo build --target=aarch64-linux-android
```

Make sure that the compiler from the NDK is in your PATH

If the library is built in debug mode, there should already be a symlink from ./target/debug/<target>/libmagical\_bitcoin\_wallet\_jni.so to the `jniLibs` directory, otherwise manually copy the shared object.
