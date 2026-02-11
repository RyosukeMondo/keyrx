# KeyRx2 Makefile
# Provides simple top-level commands for common operations

.PHONY: help build verify test launch clean setup msi e2e-auto package package-deb package-tar release sync-version generate-version installer installer-simple

# Default target - show help
.DEFAULT_GOAL := help

help: ## Show this help message
	@echo "KeyRx2 Development Commands"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  %-12s %s\n", $$1, $$2}'
	@echo ""

build: ## Build all workspace crates
	@scripts/build.sh

verify: ## Run all quality checks (clippy, fmt, tests, coverage)
	@scripts/verify.sh

test: ## Run all tests
	@scripts/test.sh

test-fast: ## Run tests with nextest (faster, parallel execution)
	@scripts/test.sh --nextest

e2e-auto: build ## Run automated E2E API tests with auto-fix
	@echo "Running automated E2E API tests..."
	@cd keyrx_ui && npm run test:e2e:auto
	@echo "E2E tests complete. Generating HTML report..."
	@cd keyrx_ui && npm run test:e2e:auto:report
	@echo "HTML report generated: keyrx_ui/report.html"

launch: ## Launch the keyrx_daemon
	@scripts/launch.sh

clean: ## Remove build artifacts and logs
	@echo "Cleaning build artifacts..."
	@rm -rf target/
	@rm -rf keyrx_ui/node_modules/
	@rm -rf keyrx_ui/dist/
	@rm -rf keyrx_ui_v2/node_modules/
	@rm -rf keyrx_ui_v2/dist/
	@rm -rf keyrx_ui_v2/src/wasm/pkg/
	@rm -rf keyrx_daemon/ui_dist/
	@rm -f scripts/logs/*.log
	@rm -rf .vite/
	@echo "Clean complete."

setup: ## Install development tools and git hooks (comprehensive setup)
	@scripts/setup_dev_environment.sh

setup-quick: ## Quick setup (no sudo, Cargo tools only)
	@echo "Quick setup: Installing Cargo development tools..."
	@command -v cargo-watch >/dev/null 2>&1 || cargo install cargo-watch
	@command -v cargo-llvm-cov >/dev/null 2>&1 || cargo install cargo-llvm-cov
	@command -v cargo-fuzz >/dev/null 2>&1 || cargo install cargo-fuzz
	@command -v cargo-nextest >/dev/null 2>&1 || cargo install cargo-nextest --locked
	@command -v wasm-pack >/dev/null 2>&1 || cargo install wasm-pack
	@echo "Installing git hooks..."
	@scripts/setup_hooks.sh
	@echo "Quick setup complete. For full setup (with system dependencies), run: make setup"

package: ## Build all Linux packages (.deb and .tar.gz)
	@scripts/package/build-all.sh

package-deb: ## Build Debian package only
	@scripts/package/build-deb.sh

package-tar: ## Build tarball with install script
	@scripts/package/build-tarball.sh

sync-version: ## Sync version from Cargo.toml to all files (SSOT)
	@scripts/sync-version.sh

generate-version: ## Generate UI version.ts with build timestamp
	@node scripts/generate-version.js

release: verify ## Create a new release (prompts for version)
	@echo "Current version: $$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')"
	@echo ""
	@read -p "Enter new version (e.g., 0.2.0): " VERSION && \
	echo "Creating release v$$VERSION..." && \
	echo "1. Update version in Cargo.toml to $$VERSION" && \
	echo "2. Update CHANGELOG.md" && \
	echo "3. Run: git add Cargo.toml CHANGELOG.md && git commit -m 'chore: bump version to $$VERSION'" && \
	echo "4. Run: git tag -a v$$VERSION -m 'Release v$$VERSION' && git push && git push origin v$$VERSION" && \
	echo "" && \
	echo "See docs/RELEASE.md for detailed instructions"

msi: ## Build Windows MSI installer (Windows only, requires WiX)
ifeq ($(OS),Windows_NT)
	@cmd /c scripts\windows\build_msi.bat
else
	@echo "MSI build is only supported on Windows"
	@echo "Use: scripts/windows/build_msi.bat (from cmd/PowerShell)"
endif

installer: ## Build self-contained Windows installer using IExpress (Windows only, no external tools)
ifeq ($(OS),Windows_NT)
	@powershell.exe -ExecutionPolicy Bypass -File scripts\installer\build-installer.ps1
else
	@echo "Installer build is only supported on Windows"
	@echo "Use: powershell.exe -ExecutionPolicy Bypass -File scripts\installer\build-installer.ps1"
endif

installer-simple: ## Build self-contained PowerShell installer (Windows only, single .ps1 file)
ifeq ($(OS),Windows_NT)
	@powershell.exe -ExecutionPolicy Bypass -File scripts\installer\create-simple-installer.ps1
else
	@echo "Installer build is only supported on Windows"
	@echo "Use: powershell.exe -ExecutionPolicy Bypass -File scripts\installer\create-simple-installer.ps1"
endif
