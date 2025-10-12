use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

/// Test utilities for integration testing
struct TestUtils {
    #[allow(dead_code)]
    project_root: PathBuf,
    binary_path: PathBuf,
    test_apk: PathBuf,
}

impl TestUtils {
    fn new() -> Self {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let binary_path = project_root.join("target/release/apkext");
        // Use local testdata directory in rust project
        let test_apk = project_root.join("testdata/sample.apk");

        Self {
            project_root,
            binary_path,
            test_apk,
        }
    }

    fn ensure_binary_exists(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.binary_path.exists() {
            return Err(format!("Binary not found at {:?}. Run: cargo build --release", self.binary_path).into());
        }
        Ok(())
    }

    fn ensure_test_apk_exists(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.test_apk.exists() {
            return Err(format!("Test APK not found at {:?}", self.test_apk).into());
        }
        Ok(())
    }

    fn ensure_java_available(&self) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("java")
            .arg("--version")
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let version_output = String::from_utf8_lossy(&output.stdout);
                println!("Java found: {}", version_output.lines().next().unwrap_or("unknown"));
                Ok(())
            }
            Ok(_) => Err("Java command failed - Java may not be properly installed".into()),
            Err(_) => Err("Java not found in PATH - Java is required for APK processing".into()),
        }
    }

    fn run_unpack(&self, apk_path: &Path, work_dir: &Path) -> Result<std::process::Output, Box<dyn std::error::Error>> {
        let apk_name = apk_path.file_name().unwrap();
        let temp_apk = work_dir.join(apk_name);

        // Copy APK to work directory to avoid race conditions between tests
        // If source doesn't exist, use the original path (for error testing)
        if apk_path.exists() {
            // Check if APK is already in work_dir to avoid copying to itself
            let target_apk = if apk_path.parent() == Some(work_dir) {
                // APK is already in work_dir, use it directly
                apk_path.to_path_buf()
            } else {
                // Copy APK to work_dir
                std::fs::copy(apk_path, &temp_apk)?;

                // Debug: verify copy was successful
                let orig_size = std::fs::metadata(apk_path)?.len();
                let copy_size = std::fs::metadata(&temp_apk)?.len();
                if orig_size != copy_size {
                    return Err(format!("Copy failed: original {} bytes, copy {} bytes", orig_size, copy_size).into());
                }
                temp_apk
            };

            // Clean up any existing output directory from previous runs
            let apk_stem = target_apk.file_stem().unwrap().to_str().unwrap();
            let output_dir = work_dir.join(apk_stem);
            if output_dir.exists() {
                std::fs::remove_dir_all(&output_dir).ok(); // Ignore errors
            }

            let output = Command::new(&self.binary_path)
                .arg("unpack")
                .arg(&target_apk)
                .current_dir(work_dir)
                .output()?;

            Ok(output)
        } else {
            // For non-existent files, run command directly (for error testing)
            let output = Command::new(&self.binary_path)
                .arg("unpack")
                .arg(apk_path)
                .current_dir(work_dir)
                .output()?;

            Ok(output)
        }
    }

    fn run_pack(&self, unpacked_dir: &Path, output_apk: &Path, work_dir: &Path) -> Result<std::process::Output, Box<dyn std::error::Error>> {
        // Clean up any existing output APK
        if output_apk.exists() {
            std::fs::remove_file(output_apk).ok(); // Ignore errors
        }

        let output = Command::new(&self.binary_path)
            .arg("pack")
            .arg(unpacked_dir)
            .arg(output_apk)
            .current_dir(work_dir)
            .output()?;

        Ok(output)
    }
}

#[test]
fn test_binary_exists() {
    let utils = TestUtils::new();
    utils.ensure_binary_exists().expect("APKext binary should exist after building");
}

#[test]
fn test_apk_exists() {
    let utils = TestUtils::new();
    utils.ensure_test_apk_exists().expect("Test APK should exist in testdata/");
}

#[test]
fn test_java_available() {
    let utils = TestUtils::new();
    utils.ensure_java_available().expect("Java should be available in system PATH");
}

#[test]
fn test_cli_help() {
    let utils = TestUtils::new();
    utils.ensure_binary_exists().expect("Binary must exist");

    let output = Command::new(&utils.binary_path)
        .arg("--help")
        .output()
        .expect("Failed to run --help command");

    assert!(output.status.success(), "Help command should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unpack"), "Help should mention unpack command");
    assert!(stdout.contains("pack"), "Help should mention pack command");
}

#[test]
fn test_unpack_basic_functionality() {
    let utils = TestUtils::new();
    utils.ensure_binary_exists().expect("Binary must exist");
    utils.ensure_test_apk_exists().expect("Test APK must exist");
    utils.ensure_java_available().expect("Java must be available");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let work_dir = temp_dir.path();

    let output = utils.run_unpack(&utils.test_apk, work_dir)
        .expect("Failed to run unpack command");

    // Print output for debugging
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("Unpack stdout: {}", stdout);
    println!("Unpack stderr: {}", stderr);

    // Unpack may fail on optional components (like dex2jar) but still succeed on main extraction
    // Check if the essential extraction was done regardless of exit code
    if !output.status.success() {
        // If command failed, check if it was just optional components failing
        if !stdout.contains("Extracting resources") && !stdout.contains("Decoding AndroidManifest.xml") {
            panic!(
                "Unpack command failed completely - no resource extraction detected!\nStdout: {}\nStderr: {}",
                stdout, stderr
            );
        }
        println!("Warning: Unpack had some failures but main extraction succeeded");
    }

    let apk_name = utils.test_apk.file_stem().unwrap().to_str().unwrap();
    // The output is created next to the APK file in work_dir
    let unpacked_base = work_dir.join(apk_name);
    let unpacked_dir = unpacked_base.join("unpacked");

    // Base directory MUST exist
    assert!(
        unpacked_base.exists(),
        "Unpack MUST create base directory: {:?}",
        unpacked_base
    );

    // Unpacked directory MUST exist
    assert!(
        unpacked_dir.exists(),
        "Unpack MUST create unpacked directory: {:?}",
        unpacked_dir
    );

    // AndroidManifest.xml MUST exist and be non-empty
    let android_manifest = unpacked_dir.join("AndroidManifest.xml");
    assert!(
        android_manifest.exists(),
        "Unpack MUST create AndroidManifest.xml: {:?}",
        android_manifest
    );

    let manifest_size = fs::metadata(&android_manifest)
        .expect("Failed to read AndroidManifest.xml metadata")
        .len();
    assert!(
        manifest_size > 100,
        "AndroidManifest.xml must be substantial (>100 bytes), got {} bytes",
        manifest_size
    );

    // apktool.yml MUST exist
    let apktool_yml = unpacked_dir.join("apktool.yml");
    assert!(
        apktool_yml.exists(),
        "Unpack MUST create apktool.yml: {:?}",
        apktool_yml
    );

    // Either classes.dex OR src/ directory with decompiled code MUST exist
    let classes_dex = unpacked_base.join("classes.dex");
    let src_dir = unpacked_base.join("src");

    let has_dex = classes_dex.exists();
    let has_src = src_dir.exists() && src_dir.is_dir();

    assert!(
        has_dex || has_src,
        "Unpack MUST create either classes.dex or src/ directory with decompiled code. Found neither at {:?} or {:?}",
        classes_dex, src_dir
    );

    if has_dex {
        let dex_size = fs::metadata(&classes_dex).unwrap().len();
        assert!(
            dex_size > 1000,
            "classes.dex should be substantial (>1KB), got {} bytes",
            dex_size
        );
        println!("✓ classes.dex extracted successfully ({} bytes)", dex_size);
    }

    if has_src {
        // Check that src directory contains some Java files (recursively)
        fn find_java_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
            let mut java_files = Vec::new();
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    java_files.extend(find_java_files(&path)?);
                } else if path.extension().and_then(|s| s.to_str()) == Some("java") {
                    java_files.push(path);
                }
            }
            Ok(java_files)
        }

        let java_files = find_java_files(&src_dir)
            .expect("Failed to search for Java files in src directory");

        assert!(
            !java_files.is_empty(),
            "src/ directory must contain at least one .java file (searched recursively)"
        );
        println!("✓ Source code decompiled successfully ({} Java files found)", java_files.len());
    }

    println!("✓ Unpack completed successfully");
}

#[test]
fn test_pack_basic_functionality() {
    let utils = TestUtils::new();
    utils.ensure_binary_exists().expect("Binary must exist");
    utils.ensure_test_apk_exists().expect("Test APK must exist");
    utils.ensure_java_available().expect("Java must be available");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let work_dir = temp_dir.path();

    // First unpack - this MUST succeed
    let unpack_output = utils.run_unpack(&utils.test_apk, work_dir)
        .expect("Failed to run unpack command");

    // Check unpack results for pack test (similar to unpack test logic)
    let unpack_stdout = String::from_utf8_lossy(&unpack_output.stdout);
    let unpack_stderr = String::from_utf8_lossy(&unpack_output.stderr);

    if !unpack_output.status.success() {
        // If command failed, check if it was just optional components failing
        if !unpack_stdout.contains("Extracting resources") && !unpack_stdout.contains("Decoding AndroidManifest.xml") {
            panic!(
                "Unpack command failed completely (required for pack test)!\nStdout: {}\nStderr: {}",
                unpack_stdout, unpack_stderr
            );
        }
        println!("Warning: Unpack had some failures but main extraction succeeded (for pack test)");
    }

    let apk_name = utils.test_apk.file_stem().unwrap().to_str().unwrap();
    // The output is created next to the APK file in work_dir
    let unpacked_base = work_dir.join(apk_name);
    let unpacked_dir = unpacked_base.join("unpacked");

    // Verify unpack structure exists
    assert!(
        unpacked_dir.exists(),
        "Unpack must create unpacked directory for pack test: {:?}",
        unpacked_dir
    );

    // Test pack functionality - this MUST succeed
    let output_apk = work_dir.join("rebuilt.apk");
    let pack_output = utils.run_pack(&unpacked_dir, &output_apk, work_dir)
        .expect("Failed to run pack command");

    if !pack_output.status.success() {
        let stdout = String::from_utf8_lossy(&pack_output.stdout);
        let stderr = String::from_utf8_lossy(&pack_output.stderr);
        panic!(
            "Pack command failed!\nStdout: {}\nStderr: {}",
            stdout, stderr
        );
    }

    // Rebuilt APK MUST exist
    assert!(
        output_apk.exists(),
        "Pack MUST create rebuilt APK file: {:?}",
        output_apk
    );

    // Verify APK file properties
    let rebuilt_size = fs::metadata(&output_apk)
        .expect("Failed to read rebuilt APK metadata")
        .len();

    assert!(
        rebuilt_size > 1000,
        "Rebuilt APK must be substantial (>1KB), got {} bytes",
        rebuilt_size
    );

    let original_size = fs::metadata(&utils.test_apk).unwrap().len();
    println!("Original APK size: {} bytes", original_size);
    println!("Rebuilt APK size: {} bytes", rebuilt_size);

    // Size should be reasonable (within 10x of original, but also not too small)
    assert!(
        rebuilt_size < original_size * 10,
        "Rebuilt APK size should be reasonable (not more than 10x original), got {} vs {}",
        rebuilt_size, original_size
    );

    assert!(
        rebuilt_size > original_size / 10,
        "Rebuilt APK size should not be suspiciously small (less than 1/10 of original), got {} vs {}",
        rebuilt_size, original_size
    );

    // Verify APK is a valid ZIP file by checking magic number
    let mut apk_file = std::fs::File::open(&output_apk)
        .expect("Failed to open rebuilt APK");
    let mut magic = [0u8; 4];
    std::io::Read::read_exact(&mut apk_file, &mut magic)
        .expect("Failed to read APK magic number");

    assert_eq!(
        &magic,
        b"PK\x03\x04",
        "Rebuilt APK must have valid ZIP magic number"
    );

    println!("✓ Pack completed successfully - valid APK created");
}

#[test]
fn test_round_trip_unpack_pack_unpack() {
    let utils = TestUtils::new();
    utils.ensure_binary_exists().expect("Binary must exist");
    utils.ensure_test_apk_exists().expect("Test APK must exist");
    utils.ensure_java_available().expect("Java must be available");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let work_dir = temp_dir.path();

    // Step 1: Unpack original APK - MUST succeed
    let unpack1_output = utils.run_unpack(&utils.test_apk, work_dir)
        .expect("Failed to run first unpack");

    // Check first unpack results
    let unpack1_stdout = String::from_utf8_lossy(&unpack1_output.stdout);
    let unpack1_stderr = String::from_utf8_lossy(&unpack1_output.stderr);

    if !unpack1_output.status.success() {
        // If command failed, check if it was just optional components failing
        if !unpack1_stdout.contains("Extracting resources") && !unpack1_stdout.contains("Decoding AndroidManifest.xml") {
            panic!(
                "First unpack failed completely in round-trip test!\nStdout: {}\nStderr: {}",
                unpack1_stdout, unpack1_stderr
            );
        }
        println!("Warning: First unpack had some failures but main extraction succeeded");
    }

    let apk_name = utils.test_apk.file_stem().unwrap().to_str().unwrap();
    // The output is created next to the APK file, not in work_dir
    let apk_parent = utils.test_apk.parent().unwrap();
    let unpacked1_base = apk_parent.join(apk_name);
    let unpacked1_dir = unpacked1_base.join("unpacked");

    assert!(
        unpacked1_dir.exists(),
        "First unpack MUST create unpacked directory: {:?}",
        unpacked1_dir
    );

    // Step 2: Pack back to APK - MUST succeed
    let rebuilt_apk = work_dir.join("rebuilt.apk");
    let pack_output = utils.run_pack(&unpacked1_dir, &rebuilt_apk, work_dir)
        .expect("Failed to run pack");

    if !pack_output.status.success() {
        let stdout = String::from_utf8_lossy(&pack_output.stdout);
        let stderr = String::from_utf8_lossy(&pack_output.stderr);
        panic!(
            "Pack failed in round-trip test!\nStdout: {}\nStderr: {}",
            stdout, stderr
        );
    }

    assert!(
        rebuilt_apk.exists(),
        "Pack MUST create rebuilt APK: {:?}",
        rebuilt_apk
    );

    // Step 3: Verify rebuilt APK before second unpack
    println!("DEBUG: rebuilt_apk path: {:?}", rebuilt_apk);
    if let Ok(metadata) = std::fs::metadata(&rebuilt_apk) {
        println!("DEBUG: rebuilt_apk size: {} bytes", metadata.len());
        if metadata.len() == 0 {
            panic!("Rebuilt APK is empty! Pack command created an empty file.");
        }
    } else {
        panic!("Rebuilt APK does not exist: {:?}", rebuilt_apk);
    }

    // Step 3: Unpack the rebuilt APK - MUST succeed
    let unpack2_output = utils.run_unpack(&rebuilt_apk, work_dir)
        .expect("Failed to run second unpack");

    // Check second unpack results
    let unpack2_stdout = String::from_utf8_lossy(&unpack2_output.stdout);
    let unpack2_stderr = String::from_utf8_lossy(&unpack2_output.stderr);

    if !unpack2_output.status.success() {
        // If command failed, check if it was just optional components failing
        if !unpack2_stdout.contains("Extracting resources") && !unpack2_stdout.contains("Decoding AndroidManifest.xml") {
            panic!(
                "Second unpack failed completely in round-trip test!\nStdout: {}\nStderr: {}",
                unpack2_stdout, unpack2_stderr
            );
        }
        println!("Warning: Second unpack had some failures but main extraction succeeded");
        println!("Second unpack stdout: {}", unpack2_stdout);
        println!("Second unpack stderr: {}", unpack2_stderr);
    }

    let rebuilt_apk_name = rebuilt_apk.file_stem().unwrap().to_str().unwrap();
    // For rebuilt APK, the output will be created next to the rebuilt APK (in work_dir)
    let unpacked2_base = work_dir.join(rebuilt_apk_name);
    let unpacked2_dir = unpacked2_base.join("unpacked");

    if !unpacked2_dir.exists() {
        println!("DEBUG: Looking for unpacked2_dir at: {:?}", unpacked2_dir);
        println!("DEBUG: work_dir contents:");
        if let Ok(entries) = std::fs::read_dir(work_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("  - {:?}", entry.path());
                }
            }
        }
        if unpacked2_base.exists() {
            println!("DEBUG: unpacked2_base contents:");
            if let Ok(entries) = std::fs::read_dir(&unpacked2_base) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        println!("  - {:?}", entry.path());
                    }
                }
            }
        }
    }

    assert!(
        unpacked2_dir.exists(),
        "Second unpack MUST create unpacked directory: {:?}",
        unpacked2_dir
    );

    // Compare key files between original and round-trip
    let manifest1 = unpacked1_dir.join("AndroidManifest.xml");
    let manifest2 = unpacked2_dir.join("AndroidManifest.xml");

    assert!(
        manifest1.exists(),
        "Original unpack MUST have AndroidManifest.xml: {:?}",
        manifest1
    );

    assert!(
        manifest2.exists(),
        "Round-trip unpack MUST have AndroidManifest.xml: {:?}",
        manifest2
    );

    let size1 = fs::metadata(&manifest1).unwrap().len();
    let size2 = fs::metadata(&manifest2).unwrap().len();

    println!("AndroidManifest.xml sizes: {} -> {}", size1, size2);

    // Sizes should be reasonably similar (within 50% difference)
    let size_diff_pct = if size1 > 0 {
        ((size2 as i64 - size1 as i64).abs() * 100) / size1 as i64
    } else {
        0
    };

    assert!(
        size_diff_pct < 50,
        "AndroidManifest.xml size difference should be reasonable (<50%), got {}% (sizes: {} vs {})",
        size_diff_pct, size1, size2
    );

    // Verify apktool.yml exists in both
    let apktool1 = unpacked1_dir.join("apktool.yml");
    let apktool2 = unpacked2_dir.join("apktool.yml");

    assert!(
        apktool1.exists(),
        "Original unpack MUST have apktool.yml: {:?}",
        apktool1
    );

    assert!(
        apktool2.exists(),
        "Round-trip unpack MUST have apktool.yml: {:?}",
        apktool2
    );

    // Verify rebuilt APK is a valid ZIP/APK
    let mut rebuilt_file = std::fs::File::open(&rebuilt_apk)
        .expect("Failed to open rebuilt APK");
    let mut magic = [0u8; 4];
    rebuilt_file.read_exact(&mut magic)
        .expect("Failed to read rebuilt APK magic number");

    assert_eq!(
        &magic,
        b"PK\x03\x04",
        "Rebuilt APK must have valid ZIP magic number"
    );

    println!("✓ Round-trip test completed successfully: original -> unpack -> pack -> unpack");
}

#[test]
fn test_error_handling_invalid_apk() {
    let utils = TestUtils::new();
    utils.ensure_binary_exists().expect("Binary must exist");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let work_dir = temp_dir.path();

    // Create a fake APK file
    let fake_apk = work_dir.join("fake.apk");
    fs::write(&fake_apk, b"This is not an APK file").unwrap();

    let output = utils.run_unpack(&fake_apk, work_dir)
        .expect("Command should execute even for invalid APK");

    // Should fail gracefully
    assert!(!output.status.success(), "Should fail for invalid APK");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty(), "Should provide error message for invalid APK");

    println!("Error message for invalid APK: {}", stderr);
}

#[test]
fn test_error_handling_missing_input() {
    let utils = TestUtils::new();
    utils.ensure_binary_exists().expect("Binary must exist");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let work_dir = temp_dir.path();

    let nonexistent_apk = work_dir.join("nonexistent.apk");

    let output = utils.run_unpack(&nonexistent_apk, work_dir)
        .expect("Command should execute even for missing file");

    // Should fail gracefully
    assert!(!output.status.success(), "Should fail for missing APK");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.is_empty(), "Should provide error message for missing APK");

    println!("Error message for missing APK: {}", stderr);
}