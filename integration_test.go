//go:build integration
// +build integration

package main

import (
	"os"
	"os/exec"
	"path/filepath"
	"testing"
)

func TestApkextUnpackIntegration(t *testing.T) {
	// Skip if testdata/sample.apk doesn't exist
	testAPK := "testdata/sample.apk"
	if _, err := os.Stat(testAPK); os.IsNotExist(err) {
		t.Skip("Test APK not found, skipping integration test")
	}

	// Build apkext binary
	buildCmd := exec.Command("go", "build", "-o", "apkext", ".")
	if err := buildCmd.Run(); err != nil {
		t.Fatalf("Failed to build apkext: %v", err)
	}
	defer os.Remove("./apkext")

	// Clean up any previous extraction
	extractDir := "testdata/sample"
	_ = os.RemoveAll(extractDir)
	defer func() {
		_ = os.RemoveAll(extractDir)
	}()

	// Test unpack command
	t.Run("UnpackCommand", func(t *testing.T) {
		cmd := exec.Command("./apkext", "unpack", testAPK)
		output, err := cmd.CombinedOutput()

		if err != nil {
			t.Fatalf("apkext unpack failed: %v\nOutput: %s", err, output)
		}

		// Verify extraction directory exists
		if _, err := os.Stat(extractDir); os.IsNotExist(err) {
			t.Error("Expected extraction directory to be created")
		}

		// Check for expected directories
		expectedDirs := []string{
			filepath.Join(extractDir, "unpacked"),
			filepath.Join(extractDir, "src"),
		}

		for _, dir := range expectedDirs {
			if _, err := os.Stat(dir); os.IsNotExist(err) {
				t.Errorf("Expected directory %s to exist", dir)
			}
		}

		// Check for AndroidManifest.xml in unpacked directory
		manifestPath := filepath.Join(extractDir, "unpacked", "AndroidManifest.xml")
		if _, err := os.Stat(manifestPath); os.IsNotExist(err) {
			t.Error("Expected AndroidManifest.xml to be extracted")
		}

		// Check for decompiled Java sources
		javaFile := filepath.Join(extractDir, "src", "io", "selendroid", "testapp", "BuildConfig.java")
		if _, err := os.Stat(javaFile); os.IsNotExist(err) {
			t.Error("Expected Java source files to be decompiled")
		}
	})

	// Test error cases
	t.Run("ErrorCases", func(t *testing.T) {
		// Test non-existent file
		cmd := exec.Command("./apkext", "unpack", "nonexistent.apk")
		_, err := cmd.CombinedOutput()
		if err == nil {
			t.Error("Expected error for non-existent file")
		}

		// Test invalid file extension
		cmd = exec.Command("./apkext", "unpack", "testdata/sample.txt")
		_, err = cmd.CombinedOutput()
		if err == nil {
			t.Error("Expected error for invalid file extension")
		}
	})
}

func TestApkextPackIntegration(t *testing.T) {
	// Skip if sample extraction doesn't exist
	extractDir := "testdata/sample"
	if _, err := os.Stat(extractDir); os.IsNotExist(err) {
		t.Skip("Extracted APK directory not found, skipping pack test")
	}

	// Build apkext binary
	buildCmd := exec.Command("go", "build", "-o", "apkext", ".")
	if err := buildCmd.Run(); err != nil {
		t.Fatalf("Failed to build apkext: %v", err)
	}
	defer os.Remove("./apkext")

	outputAPK := "testdata/repacked.apk"
	defer os.Remove(outputAPK)

	// Test pack command
	cmd := exec.Command("./apkext", "pack", extractDir, outputAPK)
	output, err := cmd.CombinedOutput()

	if err != nil {
		t.Fatalf("apkext pack failed: %v\nOutput: %s", err, output)
	}

	// Verify output APK exists
	if _, err := os.Stat(outputAPK); os.IsNotExist(err) {
		t.Error("Expected output APK to be created")
	}

	// Verify output APK is not empty
	info, err := os.Stat(outputAPK)
	if err == nil && info.Size() == 0 {
		t.Error("Output APK is empty")
	}
}
