package config

import (
	"testing"
)

func TestLoadConfig(t *testing.T) {
	cfg := Load()

	if cfg == nil {
		t.Fatal("Expected config to be non-nil")
	}

	// Test default values are set
	if cfg.Tools.ApktoolJar == "" {
		t.Error("Expected ApktoolJar to be set")
	}

	if cfg.Tools.ProcyonJar == "" {
		t.Error("Expected ProcyonJar to be set")
	}

	if cfg.Tools.Dex2JarScript == "" {
		t.Error("Expected Dex2JarScript to be set")
	}

	if cfg.JavaCmd == "" {
		t.Error("Expected JavaCmd to be set")
	}
}

func TestConfig_Colors(t *testing.T) {
	cfg := Load()

	if cfg.Colors.Green == "" {
		t.Error("Expected Green color to be set")
	}

	if cfg.Colors.Reset == "" {
		t.Error("Expected Reset color to be set")
	}
}

func TestConfig_AaptPath(t *testing.T) {
	cfg := Load()

	if cfg.Tools.AaptPath == "" {
		t.Error("Expected AaptPath to be set")
	}

	// Should contain platform-specific path
	if len(cfg.Tools.AaptPath) < 10 {
		t.Error("Expected AaptPath to be a reasonable path")
	}
}