# Tasks Document

## Task Runner & Local Development

- [x] 1. Create justfile with development recipes
  - Files: justfile (new)
  - Create task runner with setup, dev, ui, check, fmt, clippy, test recipes
  - _Leverage: existing scripts/install-hooks.sh, Cargo commands, Flutter CLI_
  - _Requirements: 1, 2_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps engineer specializing in task runners and developer experience | Task: Create justfile at project root with recipes: default (list all), setup (install tools + deps + hooks), dev (cargo watch), ui (flutter run), check (fmt + clippy + test), fmt, clippy, test, bench | Restrictions: ≤100 lines; use cd for subdirectories; include recipe descriptions; no external dependencies beyond just itself | _Leverage: scripts/install-hooks.sh, cargo commands, flutter CLI | _Requirements: 1, 2 | Success: `just` shows all commands, `just setup` installs everything, `just check` runs full quality suite._

- [x] 2. Create justfile build and release recipes
  - Files: justfile (extend)
  - Add build, build-all, build-linux, build-windows, release recipes
  - _Leverage: cargo build, cross, flutter build_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Build engineer specializing in cross-platform compilation | Task: Extend justfile with recipes: build (current platform), build-all (depends on build-linux + build-windows), build-linux (cargo --target x86_64-unknown-linux-gnu), build-windows (cross --target x86_64-pc-windows-msvc), release version (call scripts/release.sh) | Restrictions: ≤50 lines added; build-windows requires cross; release takes version parameter | _Leverage: cargo build --release, cross, flutter build | _Requirements: 2 | Success: `just build` creates release binary, `just build-all` builds both platforms._

- [x] 3. Create nextest CI profile configuration
  - Files: core/.config/nextest.toml (new)
  - Configure default and CI profiles with retries and timeouts
  - _Leverage: cargo-nextest documentation_
  - _Requirements: 4_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Test infrastructure engineer | Task: Create core/.config/nextest.toml with [profile.default] (retries=0, slow-timeout=60s) and [profile.ci] (retries=2, fail-fast=false, slow-timeout=120s) | Restrictions: ≤20 lines; follow nextest config format exactly | _Leverage: nextest documentation | _Requirements: 4 | Success: `cargo nextest run --profile ci` uses CI profile with retries._

- [x] 4. Enhance pre-commit hook with better error messages
  - Files: scripts/install-hooks.sh (modify), create hook script inline
  - Update hook to show clear pass/fail status for each check
  - _Leverage: existing scripts/install-hooks.sh_
  - _Requirements: 3_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Shell scripting engineer | Task: Modify scripts/install-hooks.sh to install enhanced pre-commit hook with: (1) echo status for each step, (2) cargo fmt --check with "Run 'cargo fmt'" hint on failure, (3) cargo clippy -- -D warnings, (4) cargo test --lib, (5) final "All pre-commit checks passed!" message | Restrictions: ≤80 lines total; fail fast on any error; colored output optional | _Leverage: existing install-hooks.sh structure | _Requirements: 3 | Success: Commits blocked on failure with clear remediation message._

## CI/CD Pipelines

- [x] 5. Enhance CI workflow with full pipeline
  - Files: .github/workflows/ci.yml (replace)
  - Implement check, test matrix, flutter tests, coverage, security, benchmark jobs
  - _Leverage: existing .github/workflows/ci.yml structure_
  - _Requirements: 4_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: CI/CD engineer specializing in GitHub Actions | Task: Replace .github/workflows/ci.yml with full pipeline: (1) check job (fmt + clippy), (2) test job with matrix [ubuntu-latest, windows-latest] using nextest, (3) test-flutter job (analyze + test --coverage), (4) coverage job (cargo-llvm-cov → Codecov), (5) security job (cargo audit), (6) benchmark job (PR only, criterion comparison) | Restrictions: ≤200 lines; use concurrency cancel-in-progress; needs: check for dependent jobs; cache with Swatinem/rust-cache | _Leverage: existing ci.yml, dtolnay/rust-toolchain, codecov/codecov-action | _Requirements: 4 | Success: All jobs run on PR, benchmark only on PR, coverage uploads to Codecov._

- [x] 6. Create release workflow for automated publishing
  - Files: .github/workflows/release.yml (new)
  - Implement matrix build, packaging, and GitHub Release publishing
  - _Leverage: softprops/action-gh-release, subosito/flutter-action_
  - _Requirements: 5_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Release engineer specializing in GitHub Actions | Task: Create .github/workflows/release.yml triggered on push tags v*: (1) build job with matrix [linux-x64, windows-x64] building Rust + Flutter, (2) package step creating tar.gz (Linux) or zip (Windows) with checksums, (3) publish job downloading artifacts and creating GitHub Release with softprops/action-gh-release | Restrictions: ≤180 lines; permissions contents:write; prerelease if tag contains '-'; use flutter-action for Flutter builds | _Leverage: softprops/action-gh-release, subosito/flutter-action | _Requirements: 5 | Success: Push v1.0.0 tag creates GitHub Release with linux + windows artifacts._

- [x] 7. Create maintenance workflow for automated updates
  - Files: .github/workflows/maintenance.yml (new)
  - Implement weekly dependency updates and security audit
  - _Leverage: peter-evans/create-pull-request, rustsec/audit-check_
  - _Requirements: 6_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps engineer specializing in maintenance automation | Task: Create .github/workflows/maintenance.yml with: (1) schedule cron '0 0 * * 0' (weekly Sunday), (2) workflow_dispatch for manual trigger, (3) update-dependencies job running cargo update + peter-evans/create-pull-request, (4) security-audit job using rustsec/audit-check | Restrictions: ≤60 lines; PR branch deps/weekly-update with delete-branch:true | _Leverage: peter-evans/create-pull-request, rustsec/audit-check | _Requirements: 6 | Success: Weekly PR created with dependency updates, audit runs and reports vulnerabilities._

## IDE & Developer Experience

- [x] 8. Create VS Code settings configuration
  - Files: .vscode/settings.json (new)
  - Configure rust-analyzer, formatOnSave, clippy as check command
  - _Leverage: rust-analyzer documentation_
  - _Requirements: 7_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: IDE configuration specialist | Task: Create .vscode/settings.json with: rust-analyzer.cargo.features="all", rust-analyzer.check.command="clippy", rust-analyzer.check.extraArgs=["--", "-D", "warnings"], editor.formatOnSave=true, [rust] defaultFormatter rust-analyzer, [dart] defaultFormatter Dart-Code | Restrictions: ≤30 lines; valid JSON; no trailing commas | _Leverage: rust-analyzer settings reference | _Requirements: 7 | Success: Opening project in VS Code shows clippy errors, saves auto-format._

- [ ] 9. Create VS Code extensions and launch configurations
  - Files: .vscode/extensions.json (new), .vscode/launch.json (new)
  - Add recommended extensions and debug configurations
  - _Leverage: VS Code documentation_
  - _Requirements: 7_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: IDE configuration specialist | Task: Create .vscode/extensions.json with recommendations [rust-lang.rust-analyzer, tamasfe.even-better-toml, serayuzgur.crates, Dart-Code.flutter, Dart-Code.dart-code]; create .vscode/launch.json with configurations: (1) "Debug keyrx run" using lldb + cargo, (2) "Debug Tests" for lib tests, (3) "Flutter" for ui/lib/main.dart | Restrictions: extensions.json ≤15 lines; launch.json ≤50 lines; use lldb type for Rust | _Leverage: VS Code launch.json schema | _Requirements: 7 | Success: VS Code suggests extensions on open, F5 launches debugger._

- [ ] 10. Create development container configuration
  - Files: .devcontainer/devcontainer.json (new)
  - Configure Rust + just with auto-setup
  - _Leverage: devcontainers/features_
  - _Requirements: 8_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Container environment engineer | Task: Create .devcontainer/devcontainer.json with: name "KeyRx Development", image mcr.microsoft.com/devcontainers/rust:1-bookworm, features for rust (stable) and just, postCreateCommand "just setup", customizations.vscode.extensions [rust-analyzer, even-better-toml, flutter], forwardPorts [8080] | Restrictions: ≤25 lines; valid JSON; use official devcontainer features | _Leverage: devcontainers feature registry | _Requirements: 8 | Success: GitHub Codespaces opens with working Rust + just environment._

## Release Management

- [ ] 11. Create git-cliff changelog configuration
  - Files: cliff.toml (new)
  - Configure conventional commit parsing and changelog format
  - _Leverage: git-cliff documentation_
  - _Requirements: 9_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Release management engineer | Task: Create cliff.toml with: [changelog] header "# Changelog\n", body template grouping by commit type; [git] conventional_commits=true, filter_unconventional=true, commit_parsers for feat→Added, fix→Fixed, perf→Performance, refactor→Changed, docs→Documentation, chore(deps)→Dependencies | Restrictions: ≤40 lines; follow git-cliff TOML schema; use Tera template syntax | _Leverage: git-cliff configuration reference | _Requirements: 9 | Success: `git-cliff` generates changelog grouped by commit type._

- [ ] 12. Create release helper script
  - Files: scripts/release.sh (new)
  - Automate version bump, changelog generation, and tag creation
  - _Leverage: git-cliff, sed, git commands_
  - _Requirements: 9_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Release automation engineer | Task: Create scripts/release.sh that: (1) accepts version as argument, (2) validates semver format, (3) updates version in core/Cargo.toml and ui/pubspec.yaml using sed, (4) runs git-cliff to generate CHANGELOG.md, (5) creates git commit "chore(release): v$VERSION", (6) creates annotated tag v$VERSION | Restrictions: ≤60 lines; executable (chmod +x); validate version format; don't push (leave to user) | _Leverage: git-cliff -o CHANGELOG.md, sed -i | _Requirements: 9 | Success: `./scripts/release.sh 1.0.0` updates versions, generates changelog, creates tag._

## Documentation & Integration

- [ ] 13. Update Cargo.toml with release profile optimizations
  - Files: core/Cargo.toml (modify)
  - Add release and profiling profiles as specified in tech.md
  - _Leverage: existing Cargo.toml_
  - _Requirements: 4, 5_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust build configuration engineer | Task: Add to core/Cargo.toml: [profile.release] with opt-level=3, lto="thin", strip=true, panic="abort", codegen-units=1; [profile.profiling] inherits="release" with debug=true, strip=false | Restrictions: ≤15 lines added; append to existing file; don't modify existing content | _Leverage: existing Cargo.toml structure | _Requirements: 4, 5 | Success: `cargo build --release` produces optimized binary, `cargo build --profile profiling` includes debug info._

- [ ] 14. Add build system documentation to README
  - Files: README.md (modify)
  - Add "Development Setup" and "Building" sections
  - _Leverage: existing README.md_
  - _Requirements: All (documentation)_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical writer | Task: Add to README.md: "## Development Setup" section with `just setup` command and prerequisites (Rust, Flutter, just); "## Building" section with `just build`, `just build-all`; "## Contributing" section with `just check` and pre-commit hooks explanation | Restrictions: ≤50 lines added; use existing README style; include code blocks for commands | _Leverage: existing README.md structure | _Requirements: All | Success: New contributor can follow README to setup and build._

- [ ] 15. Integration test and verification
  - Files: N/A (manual verification)
  - Test full workflow: setup → dev → check → build → release
  - _Leverage: all created components_
  - _Requirements: All_
  - _Prompt: Implement the task for spec build-system, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA engineer | Task: Verify build system integration: (1) run `just setup` on clean checkout, (2) run `just check` passes, (3) run `just build` creates binary, (4) test pre-commit hook blocks bad code, (5) verify CI workflow runs on push, (6) test `./scripts/release.sh 0.0.1-test` creates tag (then delete) | Restrictions: Document any issues found; create GitHub issue for any failures; clean up test artifacts | _Leverage: all build-system components | _Requirements: All | Success: All workflows function correctly end-to-end._
