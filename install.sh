#!/bin/sh

# Load configuration
script_dir="$( dirname $0 )"
if [ -f "$script_dir/.env" ]; then
    . "$script_dir/.env"
fi

cd "$script_dir"
mkdir -p "$TOOLS_DIR"
cd "$TOOLS_DIR"

if [ ! -d "$DEX2JAR_DIR" ]; then
  wget "$DEX2JAR_URL" -O "$DEX2JAR_ZIP"
  unzip "$DEX2JAR_ZIP"
  rm "$DEX2JAR_ZIP"
  chmod +x "$DEX2JAR_DIR"/*.sh
fi

if [ ! -f "$APKTOOL_JAR" ]; then
  wget "$APKTOOL_URL" -O "$APKTOOL_JAR"
  unzip "$APKTOOL_JAR" prebuilt/aapt/*
fi

if [ ! -f "$PROCYON_JAR" ]; then
  wget "$PROCYON_URL" -O "$PROCYON_JAR"
fi

