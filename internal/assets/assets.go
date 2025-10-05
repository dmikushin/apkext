package assets

//go:generate go run ../../tools/download-deps/main.go

import (
	"archive/zip"
	"embed"
	"fmt"
	"io"
	"io/fs"
	"os"
	"path/filepath"
)

//go:embed jars/*
var embeddedJars embed.FS

//go:embed tools/*
var embeddedTools embed.FS



type Manager struct {
	tempDir string
}

func NewManager() (*Manager, error) {
	tempDir, err := os.MkdirTemp("", "apkext-*")
	if err != nil {
		return nil, fmt.Errorf("failed to create temp directory: %w", err)
	}

	return &Manager{
		tempDir: tempDir,
	}, nil
}

func (m *Manager) ExtractAll() error {
	// Extract JAR files
	if err := m.extractEmbedded(embeddedJars, "jars", ""); err != nil {
		return fmt.Errorf("failed to extract JAR files: %w", err)
	}

	// Extract tools (dex2jar scripts, aapt binaries)
	if err := m.extractEmbedded(embeddedTools, "tools", ""); err != nil {
		return fmt.Errorf("failed to extract tools: %w", err)
	}

	return nil
}

func (m *Manager) GetToolsPath() string {
	return m.tempDir
}

func (m *Manager) GetJarPath(jarName string) string {
	return filepath.Join(m.tempDir, jarName)
}

func (m *Manager) GetScriptPath(scriptPath string) string {
	return filepath.Join(m.tempDir, scriptPath)
}

func (m *Manager) Cleanup() error {
	if m.tempDir != "" {
		return os.RemoveAll(m.tempDir)
	}
	return nil
}

func (m *Manager) extractEmbedded(embedFS embed.FS, srcPrefix, dstPrefix string) error {
	return fs.WalkDir(embedFS, ".", func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			return err
		}

		// Skip directories
		if d.IsDir() {
			return nil
		}

		// Read embedded file
		data, err := embedFS.ReadFile(path)
		if err != nil {
			return fmt.Errorf("failed to read embedded file %s: %w", path, err)
		}

		// Determine destination path
		relPath := path
		if srcPrefix != "" && len(path) > len(srcPrefix)+1 {
			relPath = filepath.Join(dstPrefix, path[len(srcPrefix)+1:])
		}
		destPath := filepath.Join(m.tempDir, relPath)

		// Create parent directories
		if err := os.MkdirAll(filepath.Dir(destPath), 0755); err != nil {
			return fmt.Errorf("failed to create directory %s: %w", filepath.Dir(destPath), err)
		}

		// Write file
		if err := os.WriteFile(destPath, data, 0644); err != nil {
			return fmt.Errorf("failed to write file %s: %w", destPath, err)
		}

		// Make scripts executable
		if filepath.Ext(destPath) == ".sh" {
			if err := os.Chmod(destPath, 0755); err != nil {
				return fmt.Errorf("failed to make script executable %s: %w", destPath, err)
			}
		}

		return nil
	})
}

// ExtractZipFromJar extracts specific files from a JAR (which is a ZIP file)
func (m *Manager) ExtractZipFromJar(jarPath, pattern, destDir string) error {
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
