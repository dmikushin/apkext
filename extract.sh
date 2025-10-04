#!/bin/sh

# Load configuration
script_dir="$( dirname $( readlink -f $0 ) )"
if [ -f "$script_dir/.env" ]; then
    . "$script_dir/.env"
fi

msg()
{
  echo "${COLOR_GREEN}$1${COLOR_RESET}"
}

tooldir="$script_dir/$TOOLS_DIR"

if [ $# -lt 1 ]; then
  echo "USAGE: apkext App.apk"
  echo "  Note: The name of the apk file must end with '.apk'"
  exit
fi

dname="$( dirname "$1" )";
extdir="$dname/$( basename "$1" .apk )"

if [ -d "$extdir" ]; then
  echo "Directory '$extdir' already exists."
  echo "Remove or rename it and then retry."
  exit
fi

msg "[+] Extracting under '$extdir'"

msg "[+] Extracting resources"
$JAVA_CMD -jar "$tooldir/$APKTOOL_JAR" d "$1" --frame-path "$tooldir/$FRAMEWORK_DIR" -o "$extdir/unpacked"

msg "[+] Extracting classes.dex"
unzip "$1" classes.dex -d "$extdir/"
if [ "$?" -ne "0" ]; then
  unzip "$1" class.dex -d "$extdir/"
  mv "$extdir/class.dex" "$extdir/classes.dex"
fi

msg "[+] Converting classes.dex to jar"
"$tooldir/$DEX_TOOLS_DIR/d2j-dex2jar.sh" "$extdir/classes.dex" -o "$extdir/classes.jar"
rm "$extdir/classes.dex"

msg "[+] Decompiling jar files"
rm -rf "$extdir/src"
mkdir -p "$extdir/src"
$JAVA_CMD -jar "$tooldir/$PROCYON_JAR" -jar "$extdir/classes.jar" -o "$extdir/src"

msg ""
msg "[+] Resources and smali are in '$extdir/unpacked'"
msg "[+] Decompiled classes in '$extdir/src'"

