#!/usr/bin/env bash
set -eo pipefail -o xtrace

# IMPORTANT: make sure every target is not a substring of a different one. We check for them with grep later on
BUILD_TARGETS="${BUILD_TARGETS:-linux}"

mkdir -p jvm/build/jniLibs/x86_64_linux

if echo $BUILD_TARGETS | grep "linux"; then
    cargo build --target=x86_64-unknown-linux-gnu
    cp target/x86_64-unknown-linux-gnu/debug/libbdk_jni.so jvm/build/jniLibs/x86_64_linux
fi
