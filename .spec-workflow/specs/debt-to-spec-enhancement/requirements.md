# Requirements Document

## 1. Functional Requirements

### FR1: Rename Skill from find-and-fix to debt-to-spec
- **Description**: Update skill name, description, and all references to reflect its actual purpose: converting technical debt into implementation specs
- **Acceptance Criteria**:
  - [ ] Skill directory renamed: ~/.claude/skills/debt-to-spec/
  - [ ] SKILL.md name field updated: "debt-to-spec"
  - [ ] All documentation references updated
  - [ ] Skill description accurately reflects "technical debt → spec generation"
  - [ ] Version bumped to 2.0.0
- **Priority**: HIGH
- **Rationale**: Current name "find-and-fix" implies implementation, but skill only generates specs

### FR2: Remove Auto-Implementation References
- **Description**: Remove all mentions of automatic implementation since this skill only generates specs (implementation handled by separate autonomous tools)
- **Acceptance Criteria**:
  - [ ] Remove "fix all autonomously" language
  - [ ] Remove auto-implementation workflow descriptions
  - [ ] Remove git commit automation references
  - [ ] Focus solely on spec generation quality
  - [ ] Clarify handoff to external implementation tools
- **Priority**: HIGH
- **Rationale**: Avoid confusion about skill's scope and capabilities

### FR3: Risk Classification for Task Recommendations
- **Description**: Add risk assessment to generated tasks to guide autonomous implementation tools on which tasks are safe to auto-implement
- **Acceptance Criteria**:
  - [ ] Each task includes Risk Level: LOW/MEDIUM/HIGH
  - [ ] Risk level based on: file size, complexity, test coverage, dependencies
  - [ ] Risk rationale documented per task
  - [ ] Recommendation: "Safe for autonomous implementation" or "Requires human review"
  - [ ] Metrics: effort estimate, complexity score
- **Priority**: CRITICAL
- **Rationale**: Downstream autonomous tools need to know which tasks are safe to execute without human oversight

### FR4: Dependency Graph Generation
- **Description**: Generate explicit task dependency graph to enable parallel execution by autonomous tools
- **Acceptance Criteria**:
  - [ ] Each task lists "Depends on" task IDs
  - [ ] Each task lists "Enables" task IDs (reverse dependencies)
  - [ ] Generate tasks-dependencies.json with graph structure
  - [ ] Identify tasks that can run in parallel
  - [ ] Calculate critical path through tasks
- **Priority**: HIGH
- **Rationale**: Autonomous tools can optimize execution order and parallelize independent tasks

### FR5: Validation Checklist per Task
- **Description**: Provide explicit validation checklist for autonomous tools to verify each task completion
- **Acceptance Criteria**:
  - [ ] Each task has "Validation Steps" section
  - [ ] Validation includes: build succeeds, tests pass, clippy clean, coverage maintained
  - [ ] Specific files/tests to check listed
  - [ ] Rollback criteria defined
  - [ ] Success verification commands provided
- **Priority**: HIGH
- **Rationale**: Autonomous tools need clear pass/fail criteria

### FR6: Enhanced Audit Report with Metrics
- **Description**: Generate comprehensive audit-report.md with quantitative metrics for tracking progress
- **Acceptance Criteria**:
  - [ ] Include technical debt score (0-100, 100 = clean)
  - [ ] Track violations by severity (CRITICAL, HIGH, MEDIUM, LOW)
  - [ ] Estimate lines of code to add/remove/modify
  - [ ] Estimate effort in hours per task
  - [ ] Baseline metrics for before/after comparison
  - [ ] Export metrics as JSON for tooling integration
- **Priority**: MEDIUM
- **Rationale**: Enables progress tracking and ROI measurement

### FR7: Incremental Spec Generation Mode
- **Description**: Support updating existing specs instead of always creating new ones, detecting what's already fixed
- **Acceptance Criteria**:
  - [ ] --update <spec-name> flag to update existing spec
  - [ ] Compare current codebase to previous audit
  - [ ] Mark completed tasks as [x] automatically
  - [ ] Add new violations as new tasks
  - [ ] Update audit-report.md with delta (fixed/new/remaining)
  - [ ] Preserve manual edits to requirements.md and design.md
- **Priority**: MEDIUM
- **Rationale**: Continuous improvement workflow, avoid duplicate specs

### FR8: Configurable Audit Rules
- **Description**: Allow project-specific configuration via .debt-to-spec.toml for customizing violation detection
- **Acceptance Criteria**:
  - [ ] Support .debt-to-spec.toml in project root
  - [ ] Configurable limits: max_file_size, max_function_size, min_coverage
  - [ ] Exclude paths: tests/**, benches/**, vendor/**
  - [ ] Severity overrides for specific patterns
  - [ ] Custom anti-pattern rules with regex
  - [ ] Documentation for configuration options
- **Priority**: MEDIUM
- **Rationale**: Different projects have different standards and requirements

### FR9: Multiple Output Formats
- **Description**: Generate spec in multiple formats for different autonomous tools and workflows
- **Acceptance Criteria**:
  - [ ] Markdown (default): requirements.md, design.md, tasks.md
  - [ ] JSON: spec.json with structured task data
  - [ ] YAML: spec.yaml for CI/CD tools
  - [ ] HTML: visual report with charts (optional)
  - [ ] Format selection via --format flag
  - [ ] All formats contain same information
- **Priority**: LOW
- **Rationale**: Integration with various autonomous implementation tools

### FR10: Smart Task Splitting
- **Description**: Automatically split large tasks into smaller subtasks for safer autonomous execution
- **Acceptance Criteria**:
  - [ ] Detect tasks that modify >5 files or >500 lines
  - [ ] Auto-split into sequential subtasks
  - [ ] Each subtask ≤3 files or ≤200 lines changed
  - [ ] Maintain dependency order between subtasks
  - [ ] Lower risk level for split subtasks
- **Priority**: MEDIUM
- **Rationale**: Smaller tasks are safer for autonomous execution and easier to rollback

## 2. Non-Functional Requirements

### NFR1: Performance
- **Audit Time**: Complete technical debt audit in <5 minutes for typical codebase (<100k LOC)
- **Spec Generation**: Generate all documents in <2 minutes
- **Incremental Mode**: Update existing spec in <1 minute
- **Memory Usage**: ≤2GB RAM for large codebases (500k LOC)

### NFR2: Reliability
- **Accuracy**: ≥95% of detected violations are true positives
- **Completeness**: ≥90% of actual violations detected (no false negatives)
- **Stability**: No crashes on malformed code or edge cases
- **Reproducibility**: Same input → same output (deterministic)

### NFR3: Compatibility
- **Languages**: Rust, TypeScript/JavaScript, Python, Go (minimum)
- **Platforms**: Linux, macOS, Windows
- **Claude Code**: Compatible with spec-workflow tools
- **Version Control**: Works with git repositories

### NFR4: Maintainability
- **Code Quality**: Follow all quality standards (file size ≤500 lines, coverage ≥80%)
- **Documentation**: All public functions documented with examples
- **Extensibility**: Easy to add new violation detection rules
- **Testing**: Comprehensive test suite with real codebase examples

## 3. Technical Requirements

### TR1: Dependencies
**Required Tools**:
- Claude Code CLI (spec-workflow tools integration)
- Task tool (with Explore subagent for codebase analysis)
- Read, Write, Edit, Grep, Glob tools
- Skill tool (to invoke autonomous-spec-prep)

**Optional Tools**:
- tokei (accurate line counting for file size analysis)
- git (for incremental mode delta detection)

**No External Dependencies**: Pure skill implementation, no external packages

### TR2: Input/Output

**Input**:
- Codebase path (defaults to current directory)
- Optional: spec name (defaults to generated name)
- Optional: .debt-to-spec.toml configuration
- Optional: --update flag for incremental mode

**Output**:
- .spec-workflow/specs/{spec-name}/requirements.md
- .spec-workflow/specs/{spec-name}/design.md
- .spec-workflow/specs/{spec-name}/tasks.md
- .spec-workflow/specs/{spec-name}/audit-report.md
- .spec-workflow/specs/{spec-name}/tasks-dependencies.json (new)
- .spec-workflow/specs/{spec-name}/metrics.json (new)

### TR3: Integration Points

**Integration with autonomous-spec-prep**:
- Automatically invoke after spec generation
- Include validation score in output
- Fix blockers before finalizing spec

**Integration with autonomous implementation tools**:
- Provide structured task data (JSON/YAML)
- Include risk levels and validation criteria
- Expose dependency graph for execution planning

## 4. Constraints

### C1: Scope
- **Only generates specs**: Does not implement fixes
- **Read-only analysis**: Does not modify source code during audit
- **Git-safe**: Does not create commits or branches
- **Non-interactive**: Runs autonomously without user input

### C2: Quality Gates
- **All generated specs must pass autonomous-spec-prep with ≥80 score**
- **Tasks must have detailed prompts following autonomous-spec-prep pattern**
- **Each task must have clear success criteria**
- **Risk classification required for all tasks**

### C3: Backward Compatibility
- **Must work with existing spec-workflow tools**
- **Must not break existing .spec-workflow directory structure**
- **Must preserve autonomous-spec-prep compatibility**

## 5. Success Metrics

### Spec Quality
- [ ] autonomous-spec-prep score ≥85/100 for generated specs
- [ ] 100% of tasks have risk classification
- [ ] 100% of tasks have validation criteria
- [ ] Dependency graph covers all task relationships

### Detection Accuracy
- [ ] ≥95% precision (true positives / all detections)
- [ ] ≥90% recall (detected / all actual violations)
- [ ] <5% false positive rate

### Usability
- [ ] End-to-end execution time <10 minutes
- [ ] Generated specs immediately usable by autonomous tools
- [ ] Zero manual editing required for HIGH-quality specs

### Adoption
- [ ] Compatible with 5+ autonomous implementation tools
- [ ] Supports 4+ programming languages
- [ ] Works on 10+ real-world codebases

## 6. Out of Scope

The following are explicitly NOT included:

- ❌ Automatic code implementation
- ❌ Git operations (commits, branches, PRs)
- ❌ Interactive user prompts during execution
- ❌ Real-time progress tracking during implementation
- ❌ Rollback or retry logic (handled by implementation tools)
- ❌ IDE integration or GUI
- ❌ Continuous monitoring or scheduled audits

## 7. Risk Assessment

### High Risk
- **Risk**: Audit may miss critical violations (false negatives)
- **Mitigation**: Comprehensive test suite with known violations, >90% recall target

### Medium Risk
- **Risk**: Generated specs may not be autonomous-executable
- **Mitigation**: Automatically run autonomous-spec-prep, require ≥85 score

### Low Risk
- **Risk**: Incremental mode may incorrectly mark tasks as complete
- **Mitigation**: Conservative detection, prefer false negatives over false positives
