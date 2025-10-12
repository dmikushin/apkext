# APKext Test Data

This directory contains test data for the `apkext` tool.

## Test Files

- `sample.apk` - Test APK file for unpack/pack operations (203K)

## Running Tests

Build and run the Rust integration tests:

```bash
cd .. && cargo test
```

## What Tests Verify

The integration tests check:
- **Unpack**: APK → unpacked directory structure
- **Pack**: unpacked directory → rebuilt APK
- **Round-trip**: unpack → pack → unpack cycle integrity

Tests verify that the rebuilt APK maintains proper structure and can be unpacked again successfully.