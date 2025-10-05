.PHONY: build clean download-deps deps test install help lint fmt goreleaser-check

# Load configuration from .env
include .env
export

# Build variables
BINARY_NAME=apkext
BUILD_DIR=build
ASSETS_DIR=internal/assets
JARS_DIR=$(ASSETS_DIR)/jars
TOOLS_DIR=$(ASSETS_DIR)/tools

# Version info
VERSION ?= $(shell git describe --tags --always --dirty 2>/dev/null || echo "dev")
COMMIT := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")
DATE := $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")

LDFLAGS := -X main.version=$(VERSION) -X main.commit=$(COMMIT) -X main.date=$(DATE)

# Go build flags
BUILD_FLAGS := -ldflags "$(LDFLAGS)"

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

download-deps: ## Download JAR dependencies
	@echo "Downloading JAR dependencies..."
	@mkdir -p $(JARS_DIR) $(TOOLS_DIR)

	# Download apktool
	@if [ ! -f "$(JARS_DIR)/$(APKTOOL_JAR)" ]; then \
		echo "Downloading apktool..."; \
		wget -O "$(JARS_DIR)/$(APKTOOL_JAR)" "$(APKTOOL_URL)"; \
	fi

	# Download procyon
	@if [ ! -f "$(JARS_DIR)/$(PROCYON_JAR)" ]; then \
		echo "Downloading procyon..."; \
		wget -O "$(JARS_DIR)/$(PROCYON_JAR)" "$(PROCYON_URL)"; \
	fi

	# Download and extract dex2jar
	@if [ ! -d "$(TOOLS_DIR)/$(DEX_TOOLS_DIR)" ]; then \
		echo "Downloading dex2jar..."; \
		cd $(TOOLS_DIR) && \
		for attempt in 1 2 3; do \
			echo "Download attempt $$attempt..."; \
			rm -rf "$(DEX_TOOLS_DIR)" "$(DEX2JAR_ZIP)" temp_extract_$$$$* && \
			wget -O "$(DEX2JAR_ZIP)" "$(DEX2JAR_URL)" && \
			sleep 1 && \
			if unzip -tq "$(DEX2JAR_ZIP)" >/dev/null 2>&1; then \
				echo "ZIP file integrity verified"; \
				break; \
			else \
				echo "ZIP file corrupted, retrying..."; \
				rm -f "$(DEX2JAR_ZIP)"; \
				if [ $$attempt -eq 3 ]; then \
					echo "Failed to download valid ZIP after 3 attempts"; \
					exit 1; \
				fi; \
				sleep 2; \
			fi; \
		done && \
		TEMP_DIR="temp_extract_$$$$" && \
		mkdir -p "$$TEMP_DIR" && \
		cd "$$TEMP_DIR" && \
		unzip -q "../$(DEX2JAR_ZIP)" && \
		cd .. && \
		mv "$$TEMP_DIR/$(DEX_TOOLS_DIR)" . && \
		rm -rf "$$TEMP_DIR" "$(DEX2JAR_ZIP)" && \
		chmod +x $(DEX_TOOLS_DIR)/*.sh; \
	fi

	# Extract aapt from apktool
	@if [ ! -d "$(TOOLS_DIR)/prebuilt" ]; then \
		echo "Extracting aapt from apktool..."; \
		cd $(TOOLS_DIR) && \
		unzip -oq "../jars/$(APKTOOL_JAR)" "prebuilt/*/aapt*"; \
	fi

deps: download-deps ## Install Go dependencies and download JAR files
	@echo "Installing Go dependencies..."
	@go mod download
	@go mod tidy

build: deps ## Build the binary
	@echo "Building $(BINARY_NAME)..."
	@mkdir -p $(BUILD_DIR)
	@go build $(BUILD_FLAGS) -o $(BUILD_DIR)/$(BINARY_NAME) .

test: ## Run tests
	@echo "Running tests..."
	@go test -v ./...

clean: ## Clean build artifacts and downloaded dependencies
	@echo "Cleaning..."
	@rm -rf $(BUILD_DIR)
	@rm -rf $(JARS_DIR)/*
	@rm -rf $(TOOLS_DIR)/*

install: build ## Install binary to GOPATH/bin
	@echo "Installing $(BINARY_NAME)..."
	@go install $(BUILD_FLAGS) .

# Development targets
dev-build: ## Quick build without downloading dependencies
	@echo "Quick build..."
	@mkdir -p $(BUILD_DIR)
	@go build $(BUILD_FLAGS) -o $(BUILD_DIR)/$(BINARY_NAME) .

run-unpack: build ## Run unpack command (requires APK_FILE variable)
	@if [ -z "$(APK_FILE)" ]; then \
		echo "Usage: make run-unpack APK_FILE=<path-to-apk>"; \
		exit 1; \
	fi
	@$(BUILD_DIR)/$(BINARY_NAME) unpack "$(APK_FILE)"

run-pack: build ## Run pack command (requires UNPACKED_DIR and OUTPUT_APK variables)
	@if [ -z "$(UNPACKED_DIR)" ] || [ -z "$(OUTPUT_APK)" ]; then \
		echo "Usage: make run-pack UNPACKED_DIR=<path> OUTPUT_APK=<path>"; \
		exit 1; \
	fi
	@$(BUILD_DIR)/$(BINARY_NAME) pack "$(UNPACKED_DIR)" "$(OUTPUT_APK)"

# Check if required tools are available
check-deps: ## Check if required tools are installed
	@echo "Checking dependencies..."
	@command -v go >/dev/null 2>&1 || { echo "Go is required but not installed. Aborting." >&2; exit 1; }
	@command -v java >/dev/null 2>&1 || { echo "Java is required but not installed. Aborting." >&2; exit 1; }
	@command -v wget >/dev/null 2>&1 || { echo "wget is required but not installed. Aborting." >&2; exit 1; }
	@command -v unzip >/dev/null 2>&1 || { echo "unzip is required but not installed. Aborting." >&2; exit 1; }
	@echo "All dependencies are available."

# Release targets
release: clean deps build ## Build release version
	@echo "Building release..."
	@cp $(BUILD_DIR)/$(BINARY_NAME) $(BUILD_DIR)/$(BINARY_NAME)-$(VERSION)

# Linting and formatting
lint: ## Run golangci-lint
	@echo "Running linter..."
	@golangci-lint run

fmt: ## Format Go code
	@echo "Formatting code..."
	@go fmt ./...
	@goimports -w .

# Release targets
goreleaser-check: ## Check GoReleaser configuration
	@echo "Checking GoReleaser config..."
	@goreleaser check

goreleaser-snapshot: ## Build snapshot release with GoReleaser
	@echo "Building snapshot release..."
	@goreleaser release --snapshot --clean

# Show variables
show-vars: ## Show build variables
	@echo "VERSION: $(VERSION)"
	@echo "COMMIT: $(COMMIT)"
	@echo "DATE: $(DATE)"
	@echo "BINARY_NAME: $(BINARY_NAME)"
	@echo "BUILD_DIR: $(BUILD_DIR)"