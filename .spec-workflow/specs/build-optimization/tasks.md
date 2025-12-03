# Tasks Document

## Phase 1: Dependency Audit

- [x] 1. Audit all Cargo.toml dependencies
  - Files: All `Cargo.toml` files in workspace
  - List all dependencies and their features
  - Identify unused features
  - Purpose: Understand current state
  - _Leverage: cargo tree_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer auditing deps | Task: Audit all Cargo.toml dependencies and features | Restrictions: Document all features, identify unused | _Leverage: cargo tree | Success: Complete dependency inventory | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 2. Measure baseline build metrics
  - File: Document in spec notes
  - Measure dev build time
  - Measure release build time and size
  - Purpose: Baseline for comparison
  - _Leverage: time, ls -lh_
  - _Requirements: Non-functional (baseline)_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Developer measuring builds | Task: Measure baseline build times and sizes | Restrictions: Clean builds, consistent environment | _Leverage: time command | Success: Baseline metrics documented | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Tokio Optimization

- [x] 3. Minimize tokio features
  - File: `core/Cargo.toml`
  - Replace "full" with specific features
  - Test all tokio usage still works
  - Purpose: Reduce tokio compile time
  - _Leverage: tokio docs_
  - _Requirements: 1.1, 1.3_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer optimizing deps | Task: Replace tokio "full" with minimal features | Restrictions: Only used features, test everything | _Leverage: tokio docs | Success: Tokio compiles faster, all works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 4. Minimize serde features
  - File: `core/Cargo.toml`
  - Use default-features = false
  - Add only derive if needed
  - Purpose: Reduce serde compile time
  - _Leverage: serde docs_
  - _Requirements: 1.1, 1.4_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer optimizing deps | Task: Minimize serde features | Restrictions: Only derive if needed, test serialization | _Leverage: serde docs | Success: Serde compiles faster | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 5. Minimize windows-rs features
  - File: `core/Cargo.toml`
  - List only used Windows APIs
  - Remove unused feature flags
  - Purpose: Reduce windows-rs compile time
  - _Leverage: windows-rs docs_
  - _Requirements: 1.1, 1.2_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer optimizing deps | Task: Minimize windows-rs features to used APIs only | Restrictions: Only actual API usage, test on Windows | _Leverage: windows-rs docs | Success: Windows builds faster | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Platform Feature Gates

- [x] 6. Create platform feature flags
  - File: `core/Cargo.toml`
  - Add windows-driver and linux-driver features
  - Gate platform dependencies
  - Purpose: Platform-specific builds
  - _Leverage: Cargo features_
  - _Requirements: 2.1, 2.3_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating features | Task: Create platform feature flags in Cargo.toml | Restrictions: Clear naming, proper gating | _Leverage: Cargo features | Success: Platform deps are optional | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 7. Add cfg gates to driver modules
  - Files: `core/src/drivers/{mod,windows,linux}.rs`
  - Add #[cfg] attributes to platform code
  - Ensure clean compilation on all platforms
  - Purpose: Platform isolation
  - _Leverage: Rust cfg_
  - _Requirements: 2.1, 2.2_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer adding cfg | Task: Add cfg gates to driver modules | Restrictions: Clean cross-compilation, no dead code | _Leverage: Rust cfg | Success: Cross-compile works cleanly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 8. Use target-specific dependencies
  - File: `core/Cargo.toml`
  - Use [target.'cfg(...)'.dependencies]
  - Remove conditional compilation in code where possible
  - Purpose: Cleaner dependency management
  - _Leverage: Cargo target deps_
  - _Requirements: 2.3_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer updating Cargo.toml | Task: Use target-specific dependencies | Restrictions: Match platform features, test all targets | _Leverage: Cargo target deps | Success: Dependencies platform-specific in Cargo.toml | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Build Profile Optimization

- [ ] 9. Optimize dev profile
  - File: `core/Cargo.toml`
  - Enable incremental compilation
  - Optimize dependencies in dev mode
  - Purpose: Fast dev builds
  - _Leverage: Cargo profiles_
  - _Requirements: 3.1, 3.4_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer optimizing builds | Task: Optimize dev profile for fast iteration | Restrictions: Keep debug info, fast incremental | _Leverage: Cargo profiles | Success: Dev builds significantly faster | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 10. Optimize release profile
  - File: `core/Cargo.toml`
  - Enable LTO, strip, size optimization
  - Configure codegen-units
  - Purpose: Small, fast release builds
  - _Leverage: Cargo profiles_
  - _Requirements: 4.1, 4.2, 4.3_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer optimizing release | Task: Optimize release profile for size | Restrictions: Balance size vs speed, test performance | _Leverage: Cargo profiles | Success: Release binary smaller | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 11. Add release-debug profile
  - File: `core/Cargo.toml`
  - Profile for profiling release builds
  - Keep symbols, enable optimizations
  - Purpose: Profiling support
  - _Leverage: Cargo profile inheritance_
  - _Requirements: Non-functional (debugging)_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer adding profile | Task: Add release-debug profile for profiling | Restrictions: Inherit from release, keep symbols | _Leverage: Profile inheritance | Success: Can profile release builds | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Workspace Optimization

- [ ] 12. Create workspace Cargo.toml
  - File: `Cargo.toml` (workspace root)
  - Share dependencies across workspace
  - Use resolver = "2"
  - Purpose: Consistent dependencies
  - _Leverage: Cargo workspace_
  - _Requirements: 1.1, 3.3_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer configuring workspace | Task: Create/update workspace Cargo.toml with shared deps | Restrictions: Consistent versions, resolver 2 | _Leverage: Cargo workspace | Success: Dependencies shared across crates | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 13. Enable cargo caching in CI
  - File: CI configuration (e.g., `.github/workflows/`)
  - Cache cargo registry and target
  - Use sccache or similar
  - Purpose: Fast CI builds
  - _Leverage: CI caching_
  - _Requirements: 3.1_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Developer configuring CI | Task: Enable cargo caching in CI | Restrictions: Proper cache keys, invalidation | _Leverage: CI caching | Success: CI builds use cache | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 6: Verification

- [ ] 14. Measure optimized build metrics
  - File: Document results
  - Measure new dev build time
  - Measure new release time and size
  - Purpose: Verify improvements
  - _Leverage: time, ls -lh_
  - _Requirements: Non-functional (verification)_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Developer measuring builds | Task: Measure optimized build times and sizes | Restrictions: Same conditions as baseline | _Leverage: time command | Success: Improvements documented | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 15. Test all feature combinations
  - Files: CI or local test
  - Build with minimal features
  - Build with full features
  - Purpose: Feature compatibility
  - _Leverage: cargo build --features_
  - _Requirements: 2.4_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer testing builds | Task: Test all feature combinations compile | Restrictions: All combinations, both platforms | _Leverage: cargo build | Success: All combinations build | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 16. Document feature flags
  - File: `core/README.md` or `docs/features.md`
  - Document all features and their purposes
  - Explain platform requirements
  - Purpose: Developer documentation
  - _Leverage: Implementation knowledge_
  - _Requirements: 2.4_
  - _Prompt: Implement the task for spec build-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Document all feature flags | Restrictions: Clear explanations, examples | _Leverage: Implementation | Success: Features documented for developers | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
