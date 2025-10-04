package tools

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/dmikushin/apkext/internal/assets"
	"github.com/dmikushin/apkext/internal/config"
)

type Manager struct {
	cfg         *config.Config
	assetMgr    *assets.Manager
	initialized bool
}

func NewManager(cfg *config.Config) (*Manager, error) {
	assetMgr, err := assets.NewManager()
	if err != nil {
		return nil, fmt.Errorf("failed to create asset manager: %w", err)
	}

	return &Manager{
		cfg:      cfg,
		assetMgr: assetMgr,
	}, nil
}

func (m *Manager) Initialize() error {
	if m.initialized {
		return nil
	}

	if err := m.assetMgr.ExtractAll(); err != nil {
		return fmt.Errorf("failed to extract embedded assets: %w", err)
	}

	m.initialized = true
	return nil
}

func (m *Manager) Cleanup() error {
	return m.assetMgr.Cleanup()
}

func (m *Manager) RunApktool(args ...string) error {
	if err := m.Initialize(); err != nil {
		return err
	}

	jarPath := m.assetMgr.GetJarPath(m.cfg.Tools.ApktoolJar)
	frameworkPath := m.assetMgr.GetJarPath(m.cfg.Tools.FrameworkDir)

	cmdArgs := []string{"-jar", jarPath, "--frame-path", frameworkPath}
	cmdArgs = append(cmdArgs, args...)

	cmd := exec.Command(m.cfg.JavaCmd, cmdArgs...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	return cmd.Run()
}

func (m *Manager) RunProcyon(args ...string) error {
	if err := m.Initialize(); err != nil {
		return err
	}

	jarPath := m.assetMgr.GetJarPath(m.cfg.Tools.ProcyonJar)

	cmdArgs := []string{"-jar", jarPath}
	cmdArgs = append(cmdArgs, args...)

	cmd := exec.Command(m.cfg.JavaCmd, cmdArgs...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	return cmd.Run()
}

func (m *Manager) RunDex2Jar(args ...string) error {
	if err := m.Initialize(); err != nil {
		return err
	}

	scriptPath := m.assetMgr.GetScriptPath(m.cfg.Tools.Dex2JarScript)

	cmd := exec.Command(scriptPath, args...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	return cmd.Run()
}

func (m *Manager) GetAaptPath() string {
	if err := m.Initialize(); err != nil {
		return ""
	}
	return m.assetMgr.GetScriptPath(m.cfg.Tools.AaptPath)
}

func (m *Manager) ExtractAaptFromApktool() error {
	if err := m.Initialize(); err != nil {
		return err
	}

	jarPath := m.assetMgr.GetJarPath(m.cfg.Tools.ApktoolJar)
	toolsPath := m.assetMgr.GetToolsPath()

	return m.assetMgr.ExtractZipFromJar(jarPath, "prebuilt/aapt/*", toolsPath)
}

// Utility functions
func (m *Manager) Unzip(src, dest string, pattern string) error {
	cmd := exec.Command("unzip", src, pattern, "-d", dest)
	return cmd.Run()
}

func (m *Manager) CreateDir(path string) error {
	return os.MkdirAll(path, 0755)
}

func (m *Manager) RemoveDir(path string) error {
	return os.RemoveAll(path)
}

func (m *Manager) FileExists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}

func (m *Manager) GetExtractDir(apkPath string) string {
	dir := filepath.Dir(apkPath)
	base := strings.TrimSuffix(filepath.Base(apkPath), ".apk")
	return filepath.Join(dir, base)
}

func (m *Manager) PrintMessage(msg string) {
	fmt.Printf("%s%s%s\n", m.cfg.Colors.Green, msg, m.cfg.Colors.Reset)
}