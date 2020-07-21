#!/bin/bash

set -e

# Create an Android emulator
EMU_PARAMS="-avd test -verbose -no-window -no-audio"
EMU_COMMAND="emulator"
if [[ $ABI =~ "x86" ]]; then
  EMU_COMMAND="emulator"
else
  # emulate graphics if running on ARM
  EMU_PARAMS="${EMU_PARAMS} -gpu swiftshader"
fi

# This double "sudo" monstrosity is used to have Travis execute the
# emulator with its new group permissions and help preserve the rule
# of least privilege.
sudo -E sudo -u $USER -E bash -c "${ANDROID_HOME}/emulator/${EMU_COMMAND} ${EMU_PARAMS} &"
