#!/bin/bash

# Load configuration
script_dir="$( dirname $( readlink -f $0 ) )"
if [ -f "$script_dir/.env" ]; then
    . "$script_dir/.env"
fi

tooldir="$script_dir/$TOOLS_DIR"

if [ $# -lt 2 ]; then
  echo "USAGE: apkext App/unpacked/ new.apk"
  exit
fi

dname="$( dirname $1 )";
fname="$2"

if [ "$(uname)" == "Darwin" ]; then
  aapt_path="$tooldir/prebuilt/aapt/macosx/aapt"
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
  aapt_path="$tooldir/prebuilt/aapt/linux/aapt"
fi
$JAVA_CMD -jar "$tooldir/$APKTOOL_JAR" --frame-path "$tooldir/$FRAMEWORK_DIR" \
    -aapt "$aapt_path" b "$1" -o "$2"
