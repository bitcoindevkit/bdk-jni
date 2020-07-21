#!/bin/bash

set -e

# Set up KVM
sudo apt-get update
sudo apt-get -y --no-install-recommends install libxcursor-dev bridge-utils libpulse0 libvirt-bin qemu-kvm virtinst ubuntu-vm-builder > /dev/null
# add travis user to groups
sudo adduser $USER libvirt
sudo adduser $USER kvm

sdkmanager "platforms;android-$API" >/dev/null # We need the API of the emulator we will run
sdkmanager "emulator" >/dev/null

if [[ $ABI =~ "arm" ]]; then
  # Download a pinned version of the emulator since default version can cause issues
  ${ANDROID_HOME}/emulator/emulator -version
  emulator_version=5264690 # 29.2.1.0 (build_id 5889189) ==> 28.0.23.0 (build_id 5264690)
  # sudo apt-get install -y libunwind8 libc++1
  curl -fo emulator.zip "https://dl.google.com/android/repository/emulator-linux-$emulator_version.zip"
  rm -rf "${ANDROID_HOME}/emulator"
  unzip -q emulator.zip -d "${ANDROID_HOME}"
  rm -f emulator.zip
  # install build tools and platforms for arm (to allow emulator to run)
  sdkmanager "build-tools;25.0.2" "platforms;android-25" > /dev/null
fi

sdkmanager "system-images;android-$API;$GOO;$ABI" >/dev/null # install system images for emulator

echo no | avdmanager --verbose create avd --force -n test -k "system-images;android-$API;$GOO;$ABI"
