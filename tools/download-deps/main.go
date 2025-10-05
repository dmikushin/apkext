package main

import (
	"archive/zip"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
)

const (
	DEX2JAR_URL = "https://github.com/pxb1988/dex2jar/releases/download/v2.4/dex-tools-v2.4.zip"
	APKTOOL_URL = "https://github.com/iBotPeaches/Apktool/releases/download/v2.12.1/apktool_2.12.1.jar"
	PROCYON_URL = "https://github.com/dmikushin/procyon/releases/download/v0.6.1/procyon-decompiler-v0.6.1.jar"

	APKTOOL_JAR   = "apktool.jar"
	PROCYON_JAR   = "procyon-decompiler-v0.6.1.jar"
	DEX_TOOLS_DIR = "dex-tools-v2.4"
)

func main() {
	// Find the root directory
	rootDir := findRootDir()
	assetsDir := filepath.Join(rootDir, "internal", "assets")
	jarsDir := filepath.Join(assetsDir, "jars")
	toolsDir := filepath.Join(assetsDir, "tools")

	if err := os.MkdirAll(jarsDir, 0755); err != nil {
		fmt.Printf("Failed to create jars directory: %v\n", err)
		os.Exit(1)
	}
	if err := os.MkdirAll(toolsDir, 0755); err != nil {
		fmt.Printf("Failed to create tools directory: %v\n", err)
		os.Exit(1)
	}

	// Download apktool
	apktoolPath := filepath.Join(jarsDir, APKTOOL_JAR)
	if !fileExists(apktoolPath) {
		fmt.Printf("Downloading apktool...\n")
		if err := downloadFile(APKTOOL_URL, apktoolPath); err != nil {
			fmt.Printf("Failed to download apktool: %v\n", err)
			os.Exit(1)
		}
	}

	// Download procyon
	procyonPath := filepath.Join(jarsDir, PROCYON_JAR)
	if !fileExists(procyonPath) {
		fmt.Printf("Downloading procyon...\n")
		if err := downloadFile(PROCYON_URL, procyonPath); err != nil {
			fmt.Printf("Failed to download procyon: %v\n", err)
			os.Exit(1)
		}
	}

	// Download and extract dex2jar
	dexToolsPath := filepath.Join(toolsDir, DEX_TOOLS_DIR)
	if !fileExists(dexToolsPath) {
		fmt.Printf("Downloading dex2jar...\n")
		zipPath := filepath.Join(toolsDir, "dex-tools.zip")
		if err := downloadFile(DEX2JAR_URL, zipPath); err != nil {
			fmt.Printf("Failed to download dex2jar: %v\n", err)
			os.Exit(1)
		}

		fmt.Printf("Extracting dex2jar...\n")
		if err := extractZip(zipPath, toolsDir); err != nil {
			fmt.Printf("Failed to extract dex2jar: %v\n", err)
			os.Exit(1)
		}

		// Make scripts executable
		if err := makeScriptsExecutable(dexToolsPath); err != nil {
			fmt.Printf("Failed to make scripts executable: %v\n", err)
			os.Exit(1)
		}

		// Cleanup zip file
		os.Remove(zipPath)
	}

	// Extract aapt from apktool
	aaptDir := filepath.Join(toolsDir, "prebuilt")
	if !fileExists(aaptDir) {
		fmt.Printf("Extracting aapt from apktool...\n")
		if err := extractZipPattern(apktoolPath, "prebuilt/*/aapt*", toolsDir); err != nil {
			fmt.Printf("Failed to extract aapt from apktool: %v\n", err)
			os.Exit(1)
		}
	}

	fmt.Printf("All dependencies downloaded successfully\n")
}

func downloadFile(url, filepath string) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("bad status: %s", resp.Status)
	}

	out, err := os.Create(filepath)
	if err != nil {
		return err
	}
	defer out.Close()

	_, err = io.Copy(out, resp.Body)
	return err
}

func extractZip(src, dest string) error {
	r, err := zip.OpenReader(src)
	if err != nil {
		return err
	}
	defer r.Close()

	for _, f := range r.File {
		path := filepath.Join(dest, f.Name)

		if f.FileInfo().IsDir() {
			_ = os.MkdirAll(path, f.FileInfo().Mode())
			continue
		}

		if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
			return err
		}

		rc, err := f.Open()
		if err != nil {
			return err
		}

		outFile, err := os.OpenFile(path, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, f.FileInfo().Mode())
		if err != nil {
			rc.Close()
			return err
		}

		_, err = io.Copy(outFile, rc)
		outFile.Close()
		rc.Close()

		if err != nil {
			return err
		}
	}

	return nil
}

func extractZipPattern(jarPath, pattern, destDir string) error {
	r, err := zip.OpenReader(jarPath)
	if err != nil {
		return fmt.Errorf("failed to open JAR file %s: %w", jarPath, err)
	}
	defer r.Close()

	for _, f := range r.File {
		matched, err := filepath.Match(pattern, f.Name)
		if err != nil {
			continue
		}
		if !matched {
			continue
		}

		destPath := filepath.Join(destDir, f.Name)
		if mkdirErr := os.MkdirAll(filepath.Dir(destPath), 0755); mkdirErr != nil {
			return fmt.Errorf("failed to create directory %s: %w", filepath.Dir(destPath), mkdirErr)
		}

		rc, err := f.Open()
		if err != nil {
			return fmt.Errorf("failed to open file in JAR: %w", err)
		}

		outFile, err := os.Create(destPath)
		if err != nil {
			rc.Close()
			return fmt.Errorf("failed to create output file %s: %w", destPath, err)
		}

		_, err = io.Copy(outFile, rc)
		rc.Close()
		outFile.Close()

		if err != nil {
			return fmt.Errorf("failed to copy file content: %w", err)
		}
	}

	return nil
}

func makeScriptsExecutable(dir string) error {
	return filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if strings.HasSuffix(path, ".sh") {
			return os.Chmod(path, 0755)
		}

		return nil
	})
}

func fileExists(filename string) bool {
	info, err := os.Stat(filename)
	if os.IsNotExist(err) {
		return false
	}
	return !info.IsDir()
}

func findRootDir() string {
	// Start from current directory
	dir, err := os.Getwd()
	if err != nil {
		return "."
	}

	// Look for go.mod file to identify project root
	for {
		if fileExists(filepath.Join(dir, "go.mod")) {
			return dir
		}

		parent := filepath.Dir(dir)
		if parent == dir {
			// Reached filesystem root
			break
		}
		dir = parent
	}

	// Fallback to current directory
	return "."
}
