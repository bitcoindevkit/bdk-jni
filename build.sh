#!/usr/bin/env bash
set -eo pipefail

CC="aarch64-linux-android21-clang" cargo build --target=aarch64-linux-android
CC="x86_64-linux-android21-clang" cargo build --target=x86_64-linux-android
CC="armv7a-linux-androideabi21-clang" cargo build --target=armv7-linux-androideabi
CC="i686-linux-android21-clang" cargo build --target=i686-linux-android

mkdir -p library/src/main/jniLibs/ library/src/main/jniLibs/arm64-v8a library/src/main/jniLibs/x86_64 library/src/main/jniLibs/armeabi-v7a library/src/main/jniLibs/x86

cp target/aarch64-linux-android/debug/libmagical_bitcoin_wallet_jni.so library/src/main/jniLibs/arm64-v8a
cp target/x86_64-linux-android/debug/libmagical_bitcoin_wallet_jni.so library/src/main/jniLibs/x86_64
cp target/armv7-linux-androideabi/debug/libmagical_bitcoin_wallet_jni.so library/src/main/jniLibs/armeabi-v7a
cp target/i686-linux-android/debug/libmagical_bitcoin_wallet_jni.so library/src/main/jniLibs/x86

