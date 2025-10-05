# apkext

[![Go Reference](https://pkg.go.dev/badge/github.com/dmikushin/apkext.svg)](https://pkg.go.dev/github.com/dmikushin/apkext)
[![Go Report Card](https://goreportcard.com/badge/github.com/dmikushin/apkext)](https://goreportcard.com/report/github.com/dmikushin/apkext)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Modern Go-based tool for extracting and building APK files with embedded JAR utilities.

## What this does

- Unpack APK files to extract resources, libraries, assets, and smali code
- Convert DEX bytecode to JAR format
- Decompile JAR files to readable Java source code
- Repack modified resources back into APK files

## What this uses

- [apktool](http://ibotpeaches.github.io/Apktool/) - APK resource extraction and building
- [dex2jar](https://github.com/pxb1988/dex2jar) - DEX to JAR conversion
- [procyon](https://github.com/dmikushin/procyon) - Enhanced Java decompiler with fixes

## Features

- **Single Binary**: All JAR utilities are embedded as resources - no separate installation required
- **Cross-platform**: Supports Linux and macOS
- **Modern CLI**: Clean command-line interface using Cobra
- **Improved Decompiler**: Uses dmikushin/procyon v0.6.1 with better error handling and Java 11 support

## Requirements

- **Java Runtime**: Required for running embedded JAR utilities (apktool, dex2jar, procyon)
- **Go 1.21+**: Only needed for building from source
- **Build tools**: `wget`, `unzip` (only needed for building from source)

## Installation

### Via go install (recommended)

```bash
go install github.com/dmikushin/apkext@latest
```

The binary will be installed to `$GOPATH/bin/apkext` or `~/go/bin/apkext`.

### Download pre-built binary

Download the latest release from [Releases](https://github.com/dmikushin/apkext/releases).

```bash
# Linux amd64
wget https://github.com/dmikushin/apkext/releases/latest/download/apkext_linux_x86_64.tar.gz
tar xzf apkext_linux_x86_64.tar.gz
sudo mv apkext /usr/local/bin/

# macOS
wget https://github.com/dmikushin/apkext/releases/latest/download/apkext_darwin_x86_64.tar.gz
tar xzf apkext_darwin_x86_64.tar.gz
sudo mv apkext /usr/local/bin/
```

### Via Homebrew (macOS/Linux)

```bash
brew tap dmikushin/tap
brew install apkext
```

### Building from source

```bash
# Clone the repository
git clone https://github.com/dmikushin/apkext
cd apkext

# Build (automatically downloads dependencies)
make build

# Or install to GOPATH/bin
make install
```

## Usage

### Unpack an APK file

```bash
./build/apkext unpack App.apk
```

This creates a directory `App/` containing:
- `unpacked/` - Resources, libraries, and smali code
- `src/` - Decompiled Java source code
- `classes.jar` - Converted JAR file

### Pack back to APK

```bash
./build/apkext pack App/unpacked/ NewApp.apk
```

### Get help

```bash
./build/apkext --help
./build/apkext unpack --help
./build/apkext pack --help
```

### MCP (Model Context Protocol) Integration

The tool supports MCP integration to allow AI assistants like Claude to use APK extraction capabilities.

#### Start MCP server

```bash
apkext mcp
```

#### Connect to Claude

Add the following MCP server configuration to your Claude client:

```bash
claude mcp add-json apkext -s user '{
    "command": "apkext",
    "args": ["mcp"]
}'
```

This enables Claude to:
- **unpack_apk**: Extract and decompile APK files
- **pack_apk**: Repack modified APK files

#### Available MCP tools

1. **unpack_apk**
   - **Description**: Unpack an APK file to extract resources, libraries, assets, and decompile Java source code
   - **Parameters**:
     - `apk_path` (string): Path to the APK file to unpack

2. **pack_apk**
   - **Description**: Pack an unpacked APK directory back into an APK file
   - **Parameters**:
     - `unpacked_dir` (string): Path to the unpacked APK directory (containing 'unpacked' subdirectory)
     - `output_apk` (string): Path for the output APK file

## Development

### Make targets

```bash
make help              # Show available targets
make deps              # Download dependencies and JAR files
make build             # Build the binary
make test              # Run tests
make clean             # Clean build artifacts
make check-deps        # Check if required tools are installed
```

### Development workflow

```bash
# Quick development build (without downloading deps)
make dev-build

# Test unpack with a specific APK
make run-unpack APK_FILE=example.apk

# Test pack operation
make run-pack UNPACKED_DIR=Example/unpacked OUTPUT_APK=rebuilt.apk
```

## Project Structure

```
apkext/
├── cmd/                    # Main application entry point
├── internal/
│   ├── assets/            # Embedded JAR files and tools
│   ├── config/            # Configuration management
│   └── tools/             # Tool execution and management
├── pkg/apk/               # APK processing logic
└── build/                 # Build outputs
```

## Legacy Shell Scripts

The original shell scripts (`install.sh`, `extract.sh`, `build.sh`) are still available but deprecated. The new Go binary provides the same functionality with better error handling and cross-platform support.

## Migration from Shell Scripts

- `install.sh` → `make build` (dependencies auto-downloaded)
- `extract.sh App.apk` → `apkext unpack App.apk`
- `build.sh App/unpacked/ New.apk` → `apkext pack App/unpacked/ New.apk`

## Troubleshooting

### Missing 32-bit libraries (Linux x64)

If you encounter errors with 32-bit binaries, install compatibility libraries:

```bash
sudo apt-get install lib32z1 lib32stdc++6
```

### Java not found

Ensure Java is installed and in your PATH:

```bash
java -version
which java
```

### Build issues

Check dependencies and clean build:

```bash
make check-deps
make clean
make build
```
# Trigger CI
