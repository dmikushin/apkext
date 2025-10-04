# apkext

Modern Go-based tool for extracting and building APK files with embedded JAR utilities.

## Features

- **Single Binary**: All JAR utilities are embedded as resources - no separate installation required
- **Cross-platform**: Supports Linux and macOS
- **Modern CLI**: Clean command-line interface using Cobra
- **Improved Decompiler**: Uses dmikushin/procyon v0.6.1 with better error handling and Java 11 support

## What this uses

- [apktool](http://ibotpeaches.github.io/Apktool/) - APK resource extraction and building
- [dex2jar](https://github.com/pxb1988/dex2jar) - DEX to JAR conversion
- [procyon](https://github.com/dmikushin/procyon) - Enhanced Java decompiler with fixes

## What this does

- Unpack APK files to extract resources, libraries, assets, and smali code
- Convert DEX bytecode to JAR format
- Decompile JAR files to readable Java source code
- Repack modified resources back into APK files

## Requirements

- Go 1.21+ (for building)
- Java (for running embedded JAR utilities)
- Standard Unix tools: `wget`, `unzip` (for build process)

## Installation

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
├── build/                 # Build outputs
└── Makefile               # Build automation
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
