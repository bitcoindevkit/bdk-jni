#!/usr/bin/env bash
set -eo pipefail -o xtrace

BUILD_TARGET=$(uname | tr "[:upper:]" "[:lower:]")

mkdir -p ../jvm/build/jniLibs/

if echo $BUILD_TARGET | grep "linux"; then
    cargo build --target=x86_64-unknown-linux-gnu
    cp target/x86_64-unknown-linux-gnu/debug/libbdk_jni.so ../jvm/build/jniLibs/
elif echo $BUILD_TARGET | grep "darwin"; then
    cargo build --target=x86_64-apple-darwin
    cp target/x86_64-apple-darwin/debug/libbdk_jni.dylib ../jvm/build/jniLibs/
else
    echo "Unknown jvm target $BUILD_TARGET"
    exit 1
fi
