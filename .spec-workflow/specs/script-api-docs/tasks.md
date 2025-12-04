# Tasks Document

## Phase 1: Core Types

- [x] 1. Create documentation types
  - File: `core/src/scripting/docs/types.rs`
  - Define FunctionDoc, TypeDoc, ParamDoc
  - Add Serialize derives
  - Purpose: Documentation data structures
  - _Leverage: serde_
  - _Requirements: 1.1, 2.1_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating types | Task: Create documentation types (FunctionDoc, TypeDoc, etc.) | Restrictions: Serializable, comprehensive fields | _Leverage: serde | Success: Types capture all doc info | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 2. Create DocRegistry
  - File: `core/src/scripting/docs/registry.rs`
  - Static registry for documentation
  - Registration and lookup methods
  - Purpose: Central doc storage
  - _Leverage: Static registry patterns_
  - _Requirements: 1.1, 1.3_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating registry | Task: Create DocRegistry with static storage | Restrictions: Thread-safe, efficient lookup | _Leverage: Static patterns | Success: Registry stores all docs | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 3. Create search functionality
  - File: `core/src/scripting/docs/search.rs`
  - Text search over functions and types
  - Relevance scoring
  - Purpose: Documentation search
  - _Leverage: String matching_
  - _Requirements: 4.1, 4.3_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating search | Task: Create search functionality for docs | Restrictions: Fast, relevance ranked | _Leverage: String matching | Success: Search finds relevant items | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Documentation Extraction

- [x] 4. Create rhai_doc attribute macro
  - File: `core-macros/src/rhai_doc.rs` (new crate if needed)
  - Extract doc comments and signatures
  - Register with DocRegistry
  - Purpose: Automatic doc extraction
  - _Leverage: proc-macro_
  - _Requirements: 1.1, 1.2_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Macro Developer | Task: Create rhai_doc attribute macro | Restrictions: Extract doc comments, signatures, examples | _Leverage: proc-macro | Success: Macro extracts documentation | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 5. Add documentation to existing Rhai functions
  - Files: `core/src/scripting/*.rs`
  - Add doc comments to all registered functions
  - Include parameter descriptions and examples
  - Purpose: Document existing API
  - _Leverage: rhai_doc macro_
  - _Requirements: 1.2, 3.1_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer documenting code | Task: Add doc comments to all Rhai functions | Restrictions: Complete docs, working examples | _Leverage: rhai_doc macro | Success: All functions documented | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 6. Add type documentation
  - Files: `core/src/scripting/types.rs`
  - Document all Rhai-exposed types
  - Include methods and properties
  - Purpose: Type documentation
  - _Leverage: TypeDoc type_
  - _Requirements: 2.3, 2.4_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer documenting types | Task: Add documentation for Rhai types | Restrictions: Methods, properties, examples | _Leverage: TypeDoc | Success: All types documented | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Documentation Generation

- [x] 7. Create Markdown generator
  - File: `core/src/scripting/docs/generators/markdown.rs`
  - Generate markdown from DocRegistry
  - Organized by module
  - Purpose: Readable documentation
  - _Leverage: DocRegistry_
  - _Requirements: 4.2_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating generator | Task: Create Markdown documentation generator | Restrictions: Clean formatting, module organization | _Leverage: DocRegistry | Success: Markdown docs generated | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 8. Create HTML generator
  - File: `core/src/scripting/docs/generators/html.rs`
  - Generate searchable HTML docs
  - Include syntax highlighting
  - Purpose: Web documentation
  - _Leverage: DocRegistry, templates_
  - _Requirements: 4.1, 4.2_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating generator | Task: Create HTML documentation generator | Restrictions: Searchable, syntax highlighting | _Leverage: Templates | Success: HTML docs generated with search | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 9. Create JSON schema generator
  - File: `core/src/scripting/docs/generators/json.rs`
  - Generate JSON for IDE integration
  - Include autocomplete data
  - Purpose: IDE support
  - _Leverage: serde_json_
  - _Requirements: 4.4_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating generator | Task: Create JSON schema for IDE autocomplete | Restrictions: Standard format, complete type info | _Leverage: serde_json | Success: IDEs can use JSON schema | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Example Testing

- [x] 10. Create ExampleRunner
  - File: `core/src/scripting/docs/examples.rs`
  - Run Rhai examples as tests
  - Capture and report errors
  - Purpose: Verify examples work
  - _Leverage: Rhai engine_
  - _Requirements: 3.2, 3.3_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating test runner | Task: Create ExampleRunner for doc examples | Restrictions: Run all examples, clear errors | _Leverage: Rhai engine | Success: Examples tested automatically | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 11. Add example tests to CI
  - File: CI configuration
  - Run example tests in build
  - Fail on broken examples (warning for now)
  - Purpose: Keep examples working
  - _Leverage: ExampleRunner_
  - _Requirements: 3.3, 3.4_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Developer | Task: Add example testing to CI | Restrictions: Run on every build, clear output | _Leverage: ExampleRunner | Success: CI tests examples | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Build Integration

- [x] 12. Add doc generation to build
  - File: `core/build.rs` or script
  - Generate docs on build
  - Output to docs/ directory
  - Purpose: Auto-update docs
  - _Leverage: DocGenerator_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Build Developer | Task: Add doc generation to build process | Restrictions: Fast, only on changes | _Leverage: DocGenerator | Success: Docs auto-generate | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 13. Create doc generation CLI command
  - File: CLI module
  - `keyrx docs generate` command
  - Format and output options
  - Purpose: Manual doc generation
  - _Leverage: DocGenerator_
  - _Requirements: Non-functional (usability)_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer adding CLI | Task: Add docs generate CLI command | Restrictions: Format options, output dir | _Leverage: DocGenerator | Success: CLI generates docs | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 6: Flutter Integration

- [ ] 14. Create API docs service
  - File: `ui/lib/services/api_docs_service.dart`
  - Fetch docs from Rust via FFI or file
  - Search and browse support
  - Purpose: In-app documentation
  - _Leverage: JSON docs_
  - _Requirements: 4.1_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer creating service | Task: Create API docs service for Flutter | Restrictions: Fast search, browse by module | _Leverage: JSON docs | Success: Flutter can show docs | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 15. Create docs browser widget
  - File: `ui/lib/widgets/scripting/api_browser.dart`
  - Browse and search API docs
  - Syntax highlighting for examples
  - Purpose: In-app doc viewing
  - _Leverage: API docs service_
  - _Requirements: 4.1, 4.2_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer creating widget | Task: Create API browser widget | Restrictions: Search, browse, syntax highlight | _Leverage: API docs service | Success: Users can browse docs in app | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 16. Integrate docs into script editor
  - File: `ui/lib/pages/script_editor_page.dart`
  - Add documentation panel
  - Autocomplete from docs
  - Purpose: Documentation during editing
  - _Leverage: API browser widget_
  - _Requirements: 4.4_
  - _Prompt: Implement the task for spec script-api-docs, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer integrating docs | Task: Add docs panel to script editor | Restrictions: Non-intrusive, helpful autocomplete | _Leverage: API browser | Success: Docs available while editing | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
