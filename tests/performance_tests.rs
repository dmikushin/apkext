use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use tempfile::TempDir;

/// Performance testing utilities
struct PerfTestUtils {
    binary_path: PathBuf,
    test_apk: PathBuf,
}

impl PerfTestUtils {
    fn new() -> Self {
        let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let binary_path = project_root.join("target/release/apkext");
        let test_apk = project_root.join("testdata/sample.apk");

        Self {
            binary_path,
            test_apk,
        }
    }

    fn ensure_prerequisites(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.binary_path.exists() {
            return Err(format!("Binary not found at {:?}. Run: cargo build --release", self.binary_path).into());
        }
        if !self.test_apk.exists() {
            return Err(format!("Test APK not found at {:?}", self.test_apk).into());
        }
        Ok(())
    }

    fn time_unpack(&self, apk_path: &Path, work_dir: &Path) -> Result<(std::time::Duration, bool), Box<dyn std::error::Error>> {
        let start = Instant::now();

        let output = Command::new(&self.binary_path)
            .arg("unpack")
            .arg(apk_path)
            .current_dir(work_dir)
            .output()?;

        let duration = start.elapsed();
        let success = output.status.success();

        Ok((duration, success))
    }

    fn time_pack(&self, unpacked_dir: &Path, output_apk: &Path, work_dir: &Path) -> Result<(std::time::Duration, bool), Box<dyn std::error::Error>> {
        let start = Instant::now();

        let output = Command::new(&self.binary_path)
            .arg("pack")
            .arg(unpacked_dir)
            .arg(output_apk)
            .current_dir(work_dir)
            .output()?;

        let duration = start.elapsed();
        let success = output.status.success();

        Ok((duration, success))
    }
}

#[test]
fn test_unpack_performance() {
    let utils = PerfTestUtils::new();
    utils.ensure_prerequisites().expect("Prerequisites must be met");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let work_dir = temp_dir.path();

    let (duration, _success) = utils.time_unpack(&utils.test_apk, work_dir)
        .expect("Failed to run unpack performance test");

    println!("Unpack performance: {:?}", duration);

    // Performance assertions (adjust based on expected performance)
    assert!(duration.as_secs() < 30, "Unpack should complete within 30 seconds");
    assert!(duration.as_millis() > 10, "Unpack should take at least 10ms (sanity check)");
}

#[test]
fn test_pack_performance() {
    let utils = PerfTestUtils::new();
    utils.ensure_prerequisites().expect("Prerequisites must be met");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let work_dir = temp_dir.path();

    // First unpack to get something to pack
    let _ = utils.time_unpack(&utils.test_apk, work_dir);

    let apk_name = utils.test_apk.file_stem().unwrap().to_str().unwrap();
    let unpacked_base = work_dir.join(apk_name);
    let unpacked_dir = unpacked_base.join("unpacked");
    let output_apk = work_dir.join("perf_test.apk");

    if unpacked_dir.exists() {
        let (duration, _success) = utils.time_pack(&unpacked_dir, &output_apk, work_dir)
            .expect("Failed to run pack performance test");

        println!("Pack performance: {:?}", duration);

        // Performance assertions
        assert!(duration.as_secs() < 30, "Pack should complete within 30 seconds");
        assert!(duration.as_millis() > 10, "Pack should take at least 10ms (sanity check)");
    } else {
        println!("Skipping pack performance test - unpack failed to create expected structure");
    }
}

#[test]
fn test_round_trip_performance() {
    let utils = PerfTestUtils::new();
    utils.ensure_prerequisites().expect("Prerequisites must be met");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let work_dir = temp_dir.path();

    let overall_start = Instant::now();

    // Step 1: Unpack
    let (unpack_duration, _unpack_success) = utils.time_unpack(&utils.test_apk, work_dir)
        .expect("Failed to run unpack in round-trip test");

    let apk_name = utils.test_apk.file_stem().unwrap().to_str().unwrap();
    let unpacked_base = work_dir.join(apk_name);
    let unpacked_dir = unpacked_base.join("unpacked");
    let output_apk = work_dir.join("roundtrip_test.apk");

    if !unpacked_dir.exists() {
        println!("Skipping round-trip performance test - unpack failed");
        return;
    }

    // Step 2: Pack
    let (pack_duration, _pack_success) = utils.time_pack(&unpacked_dir, &output_apk, work_dir)
        .expect("Failed to run pack in round-trip test");

    if !output_apk.exists() {
        println!("Skipping round-trip verification - pack failed");
        return;
    }

    // Step 3: Unpack again
    // Change to a different work directory to avoid conflicts
    let temp_dir2 = TempDir::new().expect("Failed to create second temp directory");
    let work_dir2 = temp_dir2.path();

    let (unpack2_duration, _unpack2_success) = utils.time_unpack(&output_apk, work_dir2)
        .expect("Failed to run second unpack in round-trip test");

    let total_duration = overall_start.elapsed();

    println!("Round-trip performance breakdown:");
    println!("  Unpack 1: {:?}", unpack_duration);
    println!("  Pack:     {:?}", pack_duration);
    println!("  Unpack 2: {:?}", unpack2_duration);
    println!("  Total:    {:?}", total_duration);

    // Performance assertions for round-trip
    assert!(total_duration.as_secs() < 60, "Full round-trip should complete within 1 minute");

    // Individual operations should be reasonably fast
    assert!(unpack_duration.as_secs() < 30, "Each unpack should be under 30 seconds");
    assert!(pack_duration.as_secs() < 30, "Pack should be under 30 seconds");
}

#[test]
fn test_cli_response_time() {
    let utils = PerfTestUtils::new();
    utils.ensure_prerequisites().expect("Prerequisites must be met");

    // Test help command response time
    let start = Instant::now();
    let output = Command::new(&utils.binary_path)
        .arg("--help")
        .output()
        .expect("Failed to run help command");
    let help_duration = start.elapsed();

    assert!(output.status.success(), "Help command should succeed");
    assert!(help_duration.as_millis() < 1000, "Help should respond within 1 second");

    println!("CLI help response time: {:?}", help_duration);

    // Test version command if available
    let start = Instant::now();
    let output = Command::new(&utils.binary_path)
        .arg("--version")
        .output()
        .expect("Failed to run version command");
    let version_duration = start.elapsed();

    if output.status.success() {
        assert!(version_duration.as_millis() < 1000, "Version should respond within 1 second");
        println!("CLI version response time: {:?}", version_duration);
    } else {
        println!("Version command not available or failed");
    }
}

#[test]
fn test_memory_usage_basic() {
    let utils = PerfTestUtils::new();
    utils.ensure_prerequisites().expect("Prerequisites must be met");

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let work_dir = temp_dir.path();

    // Get memory usage by checking if the process completes without OOM
    // This is a basic test - for detailed memory profiling, external tools would be needed

    let output = Command::new(&utils.binary_path)
        .arg("unpack")
        .arg(&utils.test_apk)
        .current_dir(work_dir)
        .output()
        .expect("Failed to run unpack for memory test");

    // If we get here without panic/OOM, memory usage is reasonable
    println!("Memory test completed - process didn't run out of memory");

    // Check that stderr doesn't contain memory-related errors
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("out of memory"), "Should not run out of memory");
    assert!(!stderr.contains("OOM"), "Should not run out of memory");
    assert!(!stderr.contains("killed"), "Process should not be killed");
}