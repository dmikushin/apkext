package mcp

import (
	"context"
	"fmt"
	"os"
	"path/filepath"

	"github.com/modelcontextprotocol/go-sdk/mcp"

	"github.com/dmikushin/apkext/internal/config"
	"github.com/dmikushin/apkext/pkg/apk"
)

// Server represents the MCP server for APK extraction tools
type Server struct {
	mcpServer *mcp.Server
	cfg       *config.Config
}

// UnpackArgs represents arguments for the unpack tool
type UnpackArgs struct {
	ApkPath string `json:"apk_path" jsonschema:"description:Path to the APK file to unpack"`
}

// PackArgs represents arguments for the pack tool
type PackArgs struct {
	UnpackedDir string `json:"unpacked_dir" jsonschema:"description:Path to the unpacked APK directory (containing 'unpacked' subdirectory)"`
	OutputApk   string `json:"output_apk" jsonschema:"description:Path for the output APK file"`
}

// NewServer creates a new MCP server instance
func NewServer() (*Server, error) {
	cfg := config.Load()

	mcpServer := mcp.NewServer(&mcp.Implementation{
		Name:    "apkext",
		Version: "1.0.0",
	}, nil)

	server := &Server{
		mcpServer: mcpServer,
		cfg:       cfg,
	}

	// Register tools
	if err := server.registerTools(); err != nil {
		return nil, fmt.Errorf("failed to register tools: %w", err)
	}

	return server, nil
}

// registerTools registers all available MCP tools
func (s *Server) registerTools() error {
	// Register unpack tool
	mcp.AddTool(s.mcpServer, &mcp.Tool{
		Name:        "unpack_apk",
		Description: "Unpack an APK file to extract resources, libraries, assets, and decompile Java source code",
	}, s.handleUnpack)

	// Register pack tool
	mcp.AddTool(s.mcpServer, &mcp.Tool{
		Name:        "pack_apk",
		Description: "Pack an unpacked APK directory back into an APK file",
	}, s.handlePack)

	return nil
}

// handleUnpack handles the unpack_apk tool call
func (s *Server) handleUnpack(ctx context.Context, req *mcp.CallToolRequest, args UnpackArgs) (*mcp.CallToolResult, any, error) {
	// Validate APK path
	apkPath := args.ApkPath
	if !filepath.IsAbs(apkPath) {
		cwd, _ := os.Getwd()
		apkPath = filepath.Join(cwd, apkPath)
	}

	if filepath.Ext(apkPath) != ".apk" {
		return &mcp.CallToolResult{
			IsError: true,
			Content: []mcp.Content{
				&mcp.TextContent{
					Text: "File must have .apk extension",
				},
			},
		}, nil, nil
	}

	if _, err := os.Stat(apkPath); os.IsNotExist(err) {
		return &mcp.CallToolResult{
			IsError: true,
			Content: []mcp.Content{
				&mcp.TextContent{
					Text: fmt.Sprintf("APK file does not exist: %s", apkPath),
				},
			},
		}, nil, nil
	}

	// Create extractor and unpack
	extractor := apk.NewExtractor(s.cfg)
	if err := extractor.Unpack(apkPath); err != nil {
		return &mcp.CallToolResult{
			IsError: true,
			Content: []mcp.Content{
				&mcp.TextContent{
					Text: fmt.Sprintf("Failed to unpack APK: %v", err),
				},
			},
		}, nil, nil
	}

	// Get extract directory name
	baseName := filepath.Base(apkPath)
	extractDir := baseName[:len(baseName)-4] // Remove .apk extension

	return &mcp.CallToolResult{
		Content: []mcp.Content{
			&mcp.TextContent{
				Text: fmt.Sprintf("Successfully unpacked %s\n\nExtracted to:\n- Resources and smali: %s/unpacked/\n- Decompiled Java source: %s/src/\n- Converted JAR: %s/classes.jar",
					apkPath, extractDir, extractDir, extractDir),
			},
		},
	}, nil, nil
}

// handlePack handles the pack_apk tool call
func (s *Server) handlePack(ctx context.Context, req *mcp.CallToolRequest, args PackArgs) (*mcp.CallToolResult, any, error) {
	// Validate paths
	unpackedDir := args.UnpackedDir
	outputApk := args.OutputApk

	if !filepath.IsAbs(unpackedDir) {
		cwd, _ := os.Getwd()
		unpackedDir = filepath.Join(cwd, unpackedDir)
	}

	if !filepath.IsAbs(outputApk) {
		cwd, _ := os.Getwd()
		outputApk = filepath.Join(cwd, outputApk)
	}

	if filepath.Ext(outputApk) != ".apk" {
		return &mcp.CallToolResult{
			IsError: true,
			Content: []mcp.Content{
				&mcp.TextContent{
					Text: "Output file must have .apk extension",
				},
			},
		}, nil, nil
	}

	if _, err := os.Stat(unpackedDir); os.IsNotExist(err) {
		return &mcp.CallToolResult{
			IsError: true,
			Content: []mcp.Content{
				&mcp.TextContent{
					Text: fmt.Sprintf("Unpacked directory does not exist: %s", unpackedDir),
				},
			},
		}, nil, nil
	}

	// Create builder and pack
	builder := apk.NewBuilder(s.cfg)
	if err := builder.Pack(unpackedDir, outputApk); err != nil {
		return &mcp.CallToolResult{
			IsError: true,
			Content: []mcp.Content{
				&mcp.TextContent{
					Text: fmt.Sprintf("Failed to pack APK: %v", err),
				},
			},
		}, nil, nil
	}

	return &mcp.CallToolResult{
		Content: []mcp.Content{
			&mcp.TextContent{
				Text: fmt.Sprintf("Successfully packed APK: %s", outputApk),
			},
		},
	}, nil, nil
}

// Run starts the MCP server
func (s *Server) Run(ctx context.Context) error {
	transport := &mcp.StdioTransport{}
	return s.mcpServer.Run(ctx, transport)
}
