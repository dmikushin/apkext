package apk

import (
	"fmt"
	"path/filepath"
	"runtime"

	"github.com/dmikushin/apkext/internal/config"
	"github.com/dmikushin/apkext/internal/tools"
)

type Builder struct {
	cfg     *config.Config
	toolMgr *tools.Manager
}

func NewBuilder(cfg *config.Config) *Builder {
	toolMgr, _ := tools.NewManager(cfg)
	return &Builder{
		cfg:     cfg,
		toolMgr: toolMgr,
	}
}

func (b *Builder) Pack(unpackedDir, outputApk string) error {
	defer b.toolMgr.Cleanup()

	// Validate input
	if !b.toolMgr.FileExists(unpackedDir) {
		return fmt.Errorf("unpacked directory does not exist: %s", unpackedDir)
	}

	if filepath.Ext(outputApk) != ".apk" {
		return fmt.Errorf("output file must have .apk extension")
	}

	b.toolMgr.PrintMessage(fmt.Sprintf("[+] Building APK from '%s' to '%s'", unpackedDir, outputApk))

	// Initialize tools and extract aapt if needed
	if err := b.toolMgr.Initialize(); err != nil {
		return fmt.Errorf("failed to initialize tools: %w", err)
	}

	if err := b.toolMgr.ExtractAaptFromApktool(); err != nil {
		return fmt.Errorf("failed to extract aapt: %w", err)
	}

	// Get platform-specific aapt path
	aaptPath := b.getAaptPath()

	// Run apktool to build APK
	return b.toolMgr.RunApktool("-aapt", aaptPath, "b", unpackedDir, "-o", outputApk)
}

func (b *Builder) getAaptPath() string {
	basePath := b.toolMgr.GetAaptPath()

	switch runtime.GOOS {
	case "darwin":
		return basePath // prebuilt/aapt/macosx/aapt
	case "linux":
		return basePath // prebuilt/aapt/linux/aapt
	default:
		return basePath // default to linux
	}
}