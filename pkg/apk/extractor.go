package apk

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/dmikushin/apkext/internal/config"
	"github.com/dmikushin/apkext/internal/tools"
)

type Extractor struct {
	cfg     *config.Config
	toolMgr *tools.Manager
}

func NewExtractor(cfg *config.Config) *Extractor {
	toolMgr, _ := tools.NewManager(cfg)
	return &Extractor{
		cfg:     cfg,
		toolMgr: toolMgr,
	}
}

func (e *Extractor) Unpack(apkPath string) error {
	defer e.toolMgr.Cleanup()

	// Validate input
	if filepath.Ext(apkPath) != ".apk" {
		return fmt.Errorf("file must have .apk extension")
	}

	if !e.toolMgr.FileExists(apkPath) {
		return fmt.Errorf("APK file does not exist: %s", apkPath)
	}

	extractDir := e.toolMgr.GetExtractDir(apkPath)
	if e.toolMgr.FileExists(extractDir) {
		return fmt.Errorf("directory '%s' already exists. Remove or rename it and then retry", extractDir)
	}

	e.toolMgr.PrintMessage(fmt.Sprintf("[+] Extracting under '%s'", extractDir))

	// Step 1: Extract resources using apktool
	if err := e.extractResources(apkPath, extractDir); err != nil {
		return fmt.Errorf("failed to extract resources: %w", err)
	}

	// Step 2: Extract classes.dex
	if err := e.extractDex(apkPath, extractDir); err != nil {
		return fmt.Errorf("failed to extract DEX: %w", err)
	}

	// Step 3: Convert DEX to JAR
	if err := e.convertDexToJar(extractDir); err != nil {
		return fmt.Errorf("failed to convert DEX to JAR: %w", err)
	}

	// Step 4: Decompile JAR to source
	if err := e.decompileJar(extractDir); err != nil {
		return fmt.Errorf("failed to decompile JAR: %w", err)
	}

	e.toolMgr.PrintMessage("")
	e.toolMgr.PrintMessage(fmt.Sprintf("[+] Resources and smali are in '%s/unpacked'", extractDir))
	e.toolMgr.PrintMessage(fmt.Sprintf("[+] Decompiled classes in '%s/src'", extractDir))

	return nil
}

func (e *Extractor) extractResources(apkPath, extractDir string) error {
	e.toolMgr.PrintMessage("[+] Extracting resources")

	unpackedDir := filepath.Join(extractDir, "unpacked")
	return e.toolMgr.RunApktool("d", apkPath, "-o", unpackedDir)
}

func (e *Extractor) extractDex(apkPath, extractDir string) error {
	e.toolMgr.PrintMessage("[+] Extracting classes.dex")

	// Try to extract classes.dex first
	err := e.toolMgr.Unzip(apkPath, extractDir, "classes.dex")
	if err != nil {
		// If classes.dex doesn't exist, try class.dex and rename it
		err = e.toolMgr.Unzip(apkPath, extractDir, "class.dex")
		if err != nil {
			return fmt.Errorf("failed to extract DEX file: %w", err)
		}

		// Rename class.dex to classes.dex
		oldPath := filepath.Join(extractDir, "class.dex")
		newPath := filepath.Join(extractDir, "classes.dex")
		if err := os.Rename(oldPath, newPath); err != nil {
			return fmt.Errorf("failed to rename class.dex to classes.dex: %w", err)
		}
	}

	return nil
}

func (e *Extractor) convertDexToJar(extractDir string) error {
	e.toolMgr.PrintMessage("[+] Converting classes.dex to jar")

	dexPath := filepath.Join(extractDir, "classes.dex")
	jarPath := filepath.Join(extractDir, "classes.jar")

	if err := e.toolMgr.RunDex2Jar(dexPath, "-o", jarPath); err != nil {
		return err
	}

	// Remove the DEX file after conversion
	return os.Remove(dexPath)
}

func (e *Extractor) decompileJar(extractDir string) error {
	e.toolMgr.PrintMessage("[+] Decompiling jar files")

	srcDir := filepath.Join(extractDir, "src")
	jarPath := filepath.Join(extractDir, "classes.jar")

	// Remove existing src directory
	if err := e.toolMgr.RemoveDir(srcDir); err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("failed to remove existing src directory: %w", err)
	}

	// Create src directory
	if err := e.toolMgr.CreateDir(srcDir); err != nil {
		return fmt.Errorf("failed to create src directory: %w", err)
	}

	// Run Procyon decompiler
	return e.toolMgr.RunProcyon("-jar", jarPath, "-o", srcDir)
}