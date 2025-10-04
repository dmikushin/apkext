package apk

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/dmikushin/apkext/internal/config"
)

func TestNewExtractor(t *testing.T) {
	cfg := &config.Config{}
	extractor := NewExtractor(cfg)

	if extractor == nil {
		t.Fatal("Expected extractor to be non-nil")
	}

	if extractor.cfg != cfg {
		t.Error("Expected extractor config to match provided config")
	}

	if extractor.toolMgr == nil {
		t.Error("Expected tool manager to be initialized")
	}
}

func TestExtractor_Unpack_InvalidExtension(t *testing.T) {
	cfg := &config.Config{}
	extractor := NewExtractor(cfg)

	err := extractor.Unpack("test.txt")
	if err == nil {
		t.Fatal("Expected error for invalid file extension")
	}

	expectedMsg := "file must have .apk extension"
	if err.Error() != expectedMsg {
		t.Errorf("Expected error message '%s', got '%s'", expectedMsg, err.Error())
	}
}

func TestExtractor_Unpack_NonExistentFile(t *testing.T) {
	cfg := &config.Config{}
	extractor := NewExtractor(cfg)

	err := extractor.Unpack("nonexistent.apk")
	if err == nil {
		t.Fatal("Expected error for non-existent file")
	}

	if err.Error() != "APK file does not exist: nonexistent.apk" {
		t.Errorf("Unexpected error message: %s", err.Error())
	}
}

func TestExtractor_Unpack_ExistingDirectory(t *testing.T) {
	// Create a temporary APK file
	tmpDir := t.TempDir()
	apkPath := filepath.Join(tmpDir, "test.apk")

	// Create empty APK file
	file, err := os.Create(apkPath)
	if err != nil {
		t.Fatal(err)
	}
	file.Close()

	// Create the expected extract directory
	extractDir := filepath.Join(tmpDir, "test")
	if err := os.MkdirAll(extractDir, 0755); err != nil {
		t.Fatal(err)
	}

	cfg := &config.Config{}
	extractor := NewExtractor(cfg)

	err = extractor.Unpack(apkPath)
	if err == nil {
		t.Fatal("Expected error for existing directory")
	}

	expectedPrefix := "directory '" + extractDir + "' already exists"
	if len(err.Error()) < len(expectedPrefix) || err.Error()[:len(expectedPrefix)] != expectedPrefix {
		t.Errorf("Expected error to start with '%s', got '%s'", expectedPrefix, err.Error())
	}
}