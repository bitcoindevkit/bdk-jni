#!/usr/bin/env bash
set -eo pipefail -o xtrace

# If ANDROID_NDK_HOME is not set then set it to github actions default
# [ -z "$ANDROID_NDK_HOME" ] && export ANDROID_NDK_HOME=$ANDROID_HOME/ndk-bundle

# Update this line accordingly if you are not building *from* darwin-x86_64 or linux-x86_64
# export PATH=$PATH:$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/`uname | tr '[:upper:]' '[:lower:]'`-x86_64/bin

# Required for 'ring' dependency to cross-compile to Android platform, must be at least 21
# export CFLAGS="-D__ANDROID_API__=21"

# IMPORTANT: make sure every target is not a substring of a different one. We check for them with grep later on
BUILD_TARGETS="${BUILD_TARGETS:-aarch64,armv7,x86_64,i686,linux}"

mkdir -p jvm/build/jniLibs/x86_64_linux

if echo $BUILD_TARGETS | grep "linux"; then
    cargo build --target=x86_64-unknown-linux-gnu
    cp target/x86_64-unknown-linux-gnu/debug/libbdk_jni.so jvm/build/jniLibs/x86_64_linux
fi
