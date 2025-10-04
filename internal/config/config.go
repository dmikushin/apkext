package config

import (
	"runtime"
)

type Config struct {
	JavaCmd string
	Colors  Colors
	Tools   Tools
}

type Colors struct {
	Green string
	Reset string
}

type Tools struct {
	ApktoolJar      string
	ProcyonJar      string
	Dex2JarScript   string
	AaptPath        string
	FrameworkDir    string
}

func Load() *Config {
	cfg := &Config{
		JavaCmd: "java",
		Colors: Colors{
			Green: "\033[1;32m",
			Reset: "\033[0;m",
		},
		Tools: Tools{
			ApktoolJar:      "apktool.jar",
			ProcyonJar:      "procyon-decompiler-v0.6.1.jar",
			Dex2JarScript:   "dex-tools-v2.4/d2j-dex2jar.sh",
			FrameworkDir:    "framework",
		},
	}

	// Set platform-specific aapt path
	switch runtime.GOOS {
	case "darwin":
		cfg.Tools.AaptPath = "prebuilt/aapt/macosx/aapt"
	case "linux":
		cfg.Tools.AaptPath = "prebuilt/aapt/linux/aapt"
	default:
		cfg.Tools.AaptPath = "prebuilt/aapt/linux/aapt"
	}

	return cfg
}