# apkext

Modern Rust-based tool for extracting and building APK files with embedded Java utilities.

## Features

- **Self-contained**: All Java dependencies (apktool, dex2jar, procyon) are embedded in the binary
- **Cross-platform**: Works on Linux, macOS, and Windows
- **Fast & Reliable**: Written in Rust with proper error handling and async operations
- **MCP Support**: Model Context Protocol server for AI assistant integration

## What this uses

- [Apktool v2.12.1](https://github.com/iBotPeaches/Apktool) - APK reverse engineering tool
- [dex2jar v2.4](https://github.com/pxb1988/dex2jar) - DEX to JAR converter
- [Procyon v0.6.1](https://github.com/dmikushin/procyon) - Java decompiler with enhanced features

## What this does

- Unpack APK files to extract resources, libraries, assets, and source code
- Convert DEX bytecode to JAR format
- Decompile Java classes to readable source code
- Repack modified sources back into APK files
- Provide MCP server for AI assistant integration

## Installation

### Prerequisites

- Rust toolchain (1.70+)
- Java Runtime Environment (JRE 8+)

### Install from GitHub

```bash
cargo install --git https://github.com/dmikushin/apkext.git
```

### Install from source

```bash
git clone https://github.com/dmikushin/apkext.git
cd apkext
cargo install --path .
```

## Usage

### Unpack APK file

```bash
apkext unpack App.apk
```

### Pack directory back to APK

```bash
apkext pack App/unpacked/ NewApp.apk
```

### Start MCP server (for AI assistants)

```bash
apkext mcp
```

## Output Structure

When you extract `Example.apk`, a directory `Example` is created:

```
Example/
├── classes.jar     # App's code converted to JAR format
├── src/           # Java source code from decompiler
└── unpacked/      # Unpacked APK contents
    ├── AndroidManifest.xml
    ├── resources.arsc
    ├── classes.dex
    ├── res/       # Resources
    ├── lib/       # Native libraries
    └── assets/    # Assets
```

## Build from Source

```bash
git clone https://github.com/dmikushin/apkext.git
cd apkext
cargo build --release
```

The build process automatically downloads and embeds all required Java tools during compilation.

## MCP Integration

This tool supports the Model Context Protocol (MCP), allowing AI assistants like Claude to use APK extraction and building capabilities. Start the MCP server with:

```bash
apkext mcp
```

## Troubleshooting

### Java-related issues

Ensure you have Java 8+ installed:

```bash
java -version
```

### 32-bit library issues on x64 Linux

If you encounter issues with native libraries:

```bash
sudo apt-get install lib32z1 lib32stdc++6
```

### Build issues

If build fails during dependency download, ensure you have internet access and try:

```bash
cargo clean
cargo build --release
```
