package apk

import (
	"testing"

	"github.com/dmikushin/apkext/internal/config"
)

func TestNewBuilder(t *testing.T) {
	cfg := &config.Config{}
	builder := NewBuilder(cfg)

	if builder == nil {
		t.Fatal("Expected builder to be non-nil")
	}

	if builder.cfg != cfg {
		t.Error("Expected builder config to match provided config")
	}

	if builder.toolMgr == nil {
		t.Error("Expected tool manager to be initialized")
	}
}

func TestBuilder_Pack_InvalidSource(t *testing.T) {
	cfg := &config.Config{}
	builder := NewBuilder(cfg)

	err := builder.Pack("nonexistent", "output.apk")
	if err == nil {
		t.Fatal("Expected error for non-existent source directory")
	}

	if err.Error() != "unpacked directory does not exist: nonexistent" {
		t.Errorf("Unexpected error message: %s", err.Error())
	}
}

func TestBuilder_Pack_InvalidOutputExtension(t *testing.T) {
	cfg := &config.Config{}
	builder := NewBuilder(cfg)

	// Create a temporary directory
	tmpDir := t.TempDir()

	err := builder.Pack(tmpDir, "output.txt")
	if err == nil {
		t.Fatal("Expected error for invalid output extension")
	}

	expectedMsg := "output file must have .apk extension"
	if err.Error() != expectedMsg {
		t.Errorf("Expected error message '%s', got '%s'", expectedMsg, err.Error())
	}
}
