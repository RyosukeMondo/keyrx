# Autonomous Development Implementation Guide

## Overview
This document provides a complete guide to implementing autonomous AI-driven development for the "Todo for Visual Thinker" project.

---

## Part 1: Critical Issues Fixed

### Original Plan Weaknesses
Your original plan had several critical issues that would prevent successful autonomous development:

1. **Infinite Loop Risk** - "Find gaps and create tasks" with no termination
2. **No Quality Gates** - Working but poor quality code would accumulate
3. **Scope Creep** - Unlimited feature generation
4. **No Error Recovery** - System would get stuck on failures
5. **Context Drift** - AI loses sight of original vision over time
6. **No Human Checkpoints** - Could build wrong thing for weeks
7. **Cost Control** - Unlimited iterations = budget overruns

### Refined Solution
The refined system includes:
- ✅ Explicit completion criteria in steering docs
- ✅ Automated quality gates (linting, tests, coverage)
- ✅ "Won't Do" lists to prevent scope creep
- ✅ Retry logic with escalation
- ✅ Milestone-based human validation
- ✅ Iteration budgets with cost control
- ✅ Decision tree with clear exit conditions

---

## Part 2: Steering Documents Created

Three comprehensive steering documents have been created in `.spec-workflow/steering/`:

### 1. product.md (19KB)
Defines the product vision with neuroscience research backing:
- **Target Users**: Visual thinkers (60-65% of population) and AI agents
- **Core Features**: Visual task board, CLI-first interface, task relationships
- **Research-Backed**: Dual Coding Theory, visual working memory, hippocampus activation
- **Won't Do List**: Explicit scope boundaries (no time tracking, no mobile apps in v1, etc.)
- **Completion Criteria**: Explicit "done" definition for v1.0
- **Success Metrics**: Measurable outcomes with research baselines
- **Implementation Roadmap**: Phased approach with priorities

**Key Innovation**: Research-backed design for visual cognition, not just "pretty UI"

### 2. design.md (16KB)
Defines UI/UX system based on 2025 trends:
- **Design Philosophy**: Enhanced minimalism, accessibility-first, AI personalization
- **2025 Trends**: WCAG 2.2 compliance, neurodivergent-friendly, microinteractions (200-500ms)
- **Visual System**: Color-coded attention capture, infinite canvas patterns, spatial UI
- **Accessibility**: Voice navigation, keyboard shortcuts, screen reader optimization
- **Performance**: 60fps animations, virtual rendering, Web Workers
- **Components**: Task cards, connection lines, microinteractions with emotional design

**Key Innovation**: Every design decision grounded in 2025 UI/UX research and accessibility standards.

### 3. tech.md (61KB) - **Comprehensive Technical Document**
Defines complete technical architecture and implementation details:
- **Stack**: Node.js, TypeScript, React, SQLite
- **Architecture**: Hexagonal (Ports & Adapters)
- **Patterns**: SOLID, DI, SSOT, SLAP
- **Quality Standards**: Max 500 lines/file, 50 lines/function, 80% coverage
- **Error Handling**: Fail fast, structured logging, custom error hierarchy
- **Performance Targets**: Specific metrics for CLI and web
- **Project Structure**: Directory organization, naming conventions, build outputs
- **CLI-First Architecture**: Complete framework for AI-first automation
- **Data Layer**: Schema validation, migrations, portability
- **Developer Experience**: Modern tooling stack (Vite, TypeScript 7.0, BiomeJS)

**Key Innovation**: Single comprehensive technical document combining architecture, infrastructure, and CLI automation framework for autonomous development.

---

## Part 3: Autonomous Development System

### Decision Tree
```
START
  ↓
Read steering documents (product.md, design.md, tech.md)
  ↓
Check tasks.md - Are there tasks?
  │
  ├─ YES → Implement next task
  │        ↓
  │        Run quality gates
  │        ↓
  │        All gates pass?
  │        ├─ YES → Commit with structured message → Log implementation
  │        └─ NO → Retry (max 3x) → Still failing? → Escalate (exit code 1)
  │
  └─ NO → All tasks complete
           ↓
           Compare codebase vs steering docs
           ↓
           Critical gaps exist?
           ├─ YES → Generate task proposals → REQUEST HUMAN APPROVAL → Approved?
           │                                                             ├─ YES → Add to tasks.md → Continue
           │                                                             └─ NO → DONE (exit code 99)
           │
           ├─ NO → Are all Phase 1 features complete?
           │        ├─ YES → DONE - Ready for UAT (exit code 99)
           │        └─ NO → Investigate why (should not happen)
           │
           └─ UNCERTAIN → Escalate to human (exit code 1)
```

### Quality Gates (All Must Pass)
Every commit must pass:
1. ESLint (linting)
2. Prettier (formatting)
3. TypeScript compiler (type checking)
4. Vitest (all tests)
5. Coverage check (≥80%)
6. Build process (successful compilation)
7. Security audit (npm audit)

### Exit Codes
- **0**: Task completed successfully, continue to next iteration
- **1**: Error encountered, escalate to human
- **2**: Checkpoint reached (every 10 tasks), request human review
- **99**: Project complete, ready for UAT

### Escalation Triggers
The AI should stop and request human help when:
- Quality gates fail 3 consecutive times
- Unclear requirement or ambiguous specification
- Architectural decision needed (not specified in tech.md)
- External dependency conflict
- Breaking change required
- Test coverage drops below threshold

---

## Part 4: Implementation Steps

### Step 1: Initial Setup (Manual - One Time)

```bash
# Initialize project
cd /home/rmondo/repos/todo-for-visual-thinker
pnpm init

# Install dependencies
pnpm add -D typescript @types/node vitest @vitest/ui \
  eslint prettier eslint-config-prettier \
  husky lint-staged

# Initialize TypeScript
pnpm tsc --init

# Setup git hooks
pnpm husky init
echo "pnpm lint-staged" > .husky/pre-commit

# Create package.json scripts
```

**package.json scripts to add:**
```json
{
  "scripts": {
    "dev": "vite",
    "dev:cli": "tsx watch src/cli/index.ts",
    "build": "tsc && vite build",
    "test": "vitest run",
    "test:watch": "vitest",
    "test:coverage": "vitest run --coverage",
    "lint": "eslint .",
    "format": "prettier --write .",
    "typecheck": "tsc --noEmit",
    "cli": "tsx src/cli/index.ts"
  },
  "lint-staged": {
    "*.{ts,tsx}": [
      "eslint --fix",
      "prettier --write",
      "vitest related --run"
    ]
  }
}
```

### Step 2: Create First Spec

```bash
# Using spec-workflow tools (if available)
# Or manually create:
mkdir -p .spec-workflow/specs/mvp-foundation
```

**Create .spec-workflow/specs/mvp-foundation/tasks.md:**
```markdown
# MVP Foundation Tasks

## Phase 1: Project Setup

### Pending
- [ ] 1. Initialize TypeScript project with proper tsconfig.json
- [ ] 2. Setup ESLint and Prettier with pre-commit hooks
- [ ] 3. Configure Vitest for testing
- [ ] 4. Create project directory structure (src/, tests/, docs/)
- [ ] 5. Setup path aliases in tsconfig.json

## Phase 2: Core Domain

### Pending
- [ ] 6. Create Task domain model (src/core/domain/Task.ts)
- [ ] 7. Create Category value object
- [ ] 8. Create Relationship domain model
- [ ] 9. Create TaskRepository port (interface)
- [ ] 10. Create Logger port (interface)

## Phase 3: Business Logic

### Pending
- [ ] 11. Implement CreateTask use case with validation
- [ ] 12. Implement UpdateTask use case
- [ ] 13. Implement DeleteTask use case
- [ ] 14. Implement QueryTasks use case
- [ ] 15. Write unit tests for all use cases (80%+ coverage)

## Phase 4: Infrastructure

### Pending
- [ ] 16. Create InMemoryTaskRepository (for testing)
- [ ] 17. Create SQLiteTaskRepository with schema migration
- [ ] 18. Create ConsoleLogger implementation
- [ ] 19. Create JSONLogger for structured logging
- [ ] 20. Write integration tests for repositories

## Phase 5: CLI

### Pending
- [ ] 21. Setup Commander.js CLI framework
- [ ] 22. Implement 'task create' command
- [ ] 23. Implement 'task list' command with JSON output
- [ ] 24. Implement 'task update' command
- [ ] 25. Implement 'task delete' command
- [ ] 26. Implement 'task status' command
- [ ] 27. Create pretty formatter for human-readable output
- [ ] 28. Write CLI integration tests

## Phase 6: Web UI (Basic)

### Pending
- [ ] 29. Setup Vite + React + TypeScript
- [ ] 30. Create TaskCard component
- [ ] 31. Create TaskForm component
- [ ] 32. Create simple list view (before canvas)
- [ ] 33. Integrate with core domain via hooks
- [ ] 34. Add Tailwind CSS styling

## Definition of Done
Each task is "done" when:
- [ ] Implementation complete
- [ ] Unit tests written and passing
- [ ] Integration tests (if applicable)
- [ ] Coverage ≥80% for changed code
- [ ] All quality gates pass
- [ ] Documentation updated (if API changed)
- [ ] Committed with structured message
```

### Step 3: Create Autonomous Development Script

**scripts/autonomous-dev.sh:**
```bash
#!/bin/bash
set -e

# Configuration
MAX_ITERATIONS=50
CHECKPOINT_INTERVAL=10
ITERATION=0
SPEC_DIR=".spec-workflow/specs/mvp-foundation"
TASKS_FILE="$SPEC_DIR/tasks.md"

echo "🤖 Starting Autonomous Development Session"
echo "Max iterations: $MAX_ITERATIONS"
echo "Checkpoint interval: $CHECKPOINT_INTERVAL"
echo ""

# Ensure spec exists
if [ ! -f "$TASKS_FILE" ]; then
  echo "❌ Error: tasks.md not found at $TASKS_FILE"
  exit 1
fi

# Main loop
while [ $ITERATION -lt $MAX_ITERATIONS ]; do
  ITERATION=$((ITERATION + 1))
  echo "========================================="
  echo "Iteration $ITERATION of $MAX_ITERATIONS"
  echo "========================================="

  # Run AI coding session
  # This would invoke Claude Code with a specific prompt
  claude-code --prompt "$(cat <<'EOF'
You are in autonomous development mode. Follow this process:

1. Read the steering documents:
   - .spec-workflow/steering/product.md (product vision, scope, roadmap)
   - .spec-workflow/steering/design.md (UI/UX system, accessibility)
   - .spec-workflow/steering/tech.md (architecture, standards, project structure)

2. Read the current tasks:
   - .spec-workflow/specs/mvp-foundation/tasks.md

3. Implement the next pending task:
   - Find first task with [ ] status
   - Implement it following tech.md standards
   - Write tests (80%+ coverage)
   - Update task status to [x] when done

4. Quality gates (all must pass):
   - Run: pnpm lint
   - Run: pnpm typecheck
   - Run: pnpm test
   - Check coverage ≥80%

5. Commit when done:
   - Use structured commit message
   - Format: "feat(scope): description\n\nTask: <task-id>\nCoverage: <percentage>"

6. Exit codes:
   - Exit 0: Success, continue
   - Exit 99: All tasks complete
   - Exit 1: Error, escalate
   - Exit 2: Need human review

If quality gates fail 3 times, exit 1.
If tasks.md has no pending tasks, exit 99.
EOF
)"

  EXIT_CODE=$?

  # Handle exit codes
  case $EXIT_CODE in
    0)
      echo "✅ Iteration $ITERATION completed successfully"
      ;;
    99)
      echo "🎉 Project complete! Ready for UAT"
      exit 0
      ;;
    1)
      echo "❌ Error encountered. Escalating to human..."
      exit 1
      ;;
    2)
      echo "⏸️  Human review requested"
      exit 2
      ;;
    *)
      echo "❓ Unknown exit code: $EXIT_CODE"
      exit 1
      ;;
  esac

  # Checkpoint every N iterations
  if [ $((ITERATION % CHECKPOINT_INTERVAL)) -eq 0 ]; then
    echo ""
    echo "📊 Checkpoint $((ITERATION / CHECKPOINT_INTERVAL)) reached"
    echo "Requesting human review..."
    # Could create GitHub issue, send notification, etc.
    exit 2
  fi

  echo ""
done

echo "⚠️  Max iterations ($MAX_ITERATIONS) reached. Human review required."
exit 2
```

Make it executable:
```bash
chmod +x scripts/autonomous-dev.sh
```

### Step 4: Create Autonomous Development Prompt Template

**.claude/prompts/autonomous-dev.md:**
```markdown
# Autonomous Development Mode

## Mission
You are implementing features for "Todo for Visual Thinker" autonomously. Your goal is to complete one atomic task per session following strict quality standards.

## Process

### 1. Context Loading
Read these files to understand the project:
- `.spec-workflow/steering/product.md` - Product vision, scope, and roadmap
- `.spec-workflow/steering/design.md` - UI/UX system and accessibility guidelines
- `.spec-workflow/steering/tech.md` - Complete technical architecture, standards, and project structure
- `.spec-workflow/specs/mvp-foundation/tasks.md` - Current task list

### 2. Task Selection
- Find the first task marked with `[ ]` (pending)
- If no pending tasks exist:
  - Check if all Phase 1 features are complete (see product.md)
  - If complete: Exit with code 99 (project done)
  - If not complete: Analyze gaps, propose new tasks, request human approval

### 3. Implementation
Follow tech.md standards:
- Max 500 lines per file (excluding comments/blank lines)
- Max 50 lines per function
- Dependency injection for all external dependencies
- SOLID principles, hexagonal architecture
- Fail-fast error handling

### 4. Testing
Write tests achieving:
- 80%+ coverage minimum
- 90%+ for critical paths
- Unit tests for business logic
- Integration tests for adapters

### 5. Quality Gates
Run these commands (all must pass):
```bash
pnpm lint           # ESLint
pnpm typecheck      # TypeScript
pnpm test           # Tests
pnpm test:coverage  # Coverage check
pnpm build          # Build verification
```

If any gate fails:
- Fix the issue
- Retry (max 3 attempts)
- After 3 failures: Exit with code 1 (escalate)

### 6. Commit
When all quality gates pass:
```bash
git add .
git commit -m "$(cat <<'COMMIT_MSG'
<type>(<scope>): <description>

<detailed explanation of what changed and why>

Task: <task-id from tasks.md>
Coverage: <test coverage percentage>

🤖 Generated with Claude Code
COMMIT_MSG
)"
```

Commit types: feat, fix, refactor, test, docs, chore

### 7. Update Tasks
- Mark completed task with `[x]` in tasks.md
- Commit the tasks.md update

### 8. Exit
- Exit 0: Task completed successfully
- Exit 99: All tasks complete, ready for UAT
- Exit 1: Error, need human help
- Exit 2: Checkpoint, request review

## Quality Standards

### Code Metrics
- Lines per file: ≤500 (excluding comments)
- Lines per function: ≤50
- Test coverage: ≥80% (≥90% critical)
- Cyclomatic complexity: ≤10

### Architecture
- Hexagonal (Ports & Adapters)
- Dependency injection mandatory
- SOLID principles
- Single Level of Abstraction Principle

### Error Handling
- Fail fast: validate at entry
- Custom error hierarchy
- Structured JSON logging
- Never log secrets/PII

### Testing
- Unit tests: Isolated, mocked dependencies
- Integration tests: Real dependencies
- E2E tests: Critical user flows only

## Escalation Triggers
Exit code 1 (request human help) when:
- Quality gates fail 3 times
- Unclear requirement
- Architectural decision needed
- External dependency conflict
- Breaking change required

## Success Criteria
This session succeeds when:
- [x] One atomic task completed
- [x] All quality gates passed
- [x] Tests written and passing (≥80% coverage)
- [x] Code follows all standards
- [x] Commit created with structured message
- [x] tasks.md updated

## Current Status
- Iteration: [Will be injected by script]
- Spec: mvp-foundation
- Next Task: [Read from tasks.md]

---

Begin implementation. Read steering docs, find next task, implement, test, commit. Good luck! 🚀
```

---

## Part 5: Running Autonomous Development

### Manual Execution (Testing)
```bash
# Run one iteration manually
claude-code --file .claude/prompts/autonomous-dev.md

# Check results
git log -1
cat .spec-workflow/specs/mvp-foundation/tasks.md
```

### Automated Execution (Full Autonomous)
```bash
# Run the automation script
./scripts/autonomous-dev.sh

# It will:
# 1. Run AI sessions in a loop
# 2. Stop at checkpoints (every 10 tasks)
# 3. Stop on errors
# 4. Stop when complete
```

### Monitoring Progress
```bash
# Check which tasks are done
grep -E "\[.\]" .spec-workflow/specs/mvp-foundation/tasks.md

# Check commit history
git log --oneline

# Check test coverage
pnpm test:coverage

# Check code quality
pnpm lint
pnpm typecheck
```

---

## Part 6: Human Checkpoints

### Checkpoint Protocol
Every 10 completed tasks, the system pauses and requests human review.

**What to Review:**
1. **Code Quality**
   - Are patterns consistent?
   - Is code readable and maintainable?
   - Are tests meaningful?

2. **Architecture**
   - Does it follow tech.md?
   - Are dependencies properly injected?
   - Is separation of concerns maintained?

3. **Direction**
   - Are we building the right thing?
   - Is scope creeping?
   - Should we adjust priorities?

4. **Performance**
   - Are metrics within targets?
   - Any performance regressions?

**Decision Points:**
- ✅ **Continue**: Run next 10 iterations
- 🔄 **Adjust**: Update steering docs, restart
- ⏸️ **Pause**: Take over manually
- ✋ **Stop**: Project complete or pivot needed

---

## Part 7: Completion Criteria

### v1.0 Definition of Done
From product.md, v1.0 is complete when:

- [ ] All Phase 1 features implemented
- [ ] CLI fully functional with JSON output
- [ ] Web UI with visual task board
- [ ] Tasks CRUD operations work
- [ ] Visual relationships (drag-drop connections)
- [ ] Data persistence (SQLite)
- [ ] 80%+ test coverage
- [ ] All quality gates passing
- [ ] Performance targets met (<100ms ops, <1s render)
- [ ] WCAG 2.1 AA compliant
- [ ] Documentation complete
- [ ] Build pipeline working
- [ ] No critical bugs

### UAT (User Acceptance Testing)
When all criteria met:
1. Create UAT test plan
2. Recruit visual thinkers for testing
3. Gather feedback
4. Fix critical issues
5. Release v1.0

---

## Part 8: Troubleshooting

### AI Gets Stuck
**Symptom**: Same task fails repeatedly
**Solution**:
- Check error logs
- Simplify task (break into smaller pieces)
- Update tech.md with more specific guidance
- Manually fix blocking issue, commit, resume

### Scope Creep
**Symptom**: AI keeps adding features beyond MVP
**Solution**:
- Review "Won't Do" list in product.md
- Make it more explicit
- Emphasize completion criteria
- Manually edit tasks.md to remove out-of-scope tasks

### Quality Degradation
**Symptom**: Code quality declining over time
**Solution**:
- Review tech.md standards
- Add more specific examples
- Increase checkpoint frequency
- Add automated metrics checks

### Context Drift
**Symptom**: New code doesn't match existing patterns
**Solution**:
- Add Architecture Decision Records (ADRs)
- Document key patterns in tech.md
- Provide code examples in steering docs
- More frequent checkpoints

### Budget Overrun
**Symptom**: Iterations exceeding budget
**Solution**:
- Reduce MAX_ITERATIONS
- Smaller task breakdown
- Focus on critical path only
- Manual implementation of complex features

---

## Part 9: Next Steps

### Immediate (Do Now)
1. ✅ Review steering documents (product.md, design.md, tech.md)
2. ⏭️ Run Step 1 (Initial Setup) manually
3. ⏭️ Create first tasks.md (use template provided)
4. ⏭️ Test one manual iteration
5. ⏭️ Create and test autonomous-dev.sh script

### Short-term (This Week)
1. Complete Phase 1 tasks (project setup)
2. Complete Phase 2 tasks (core domain)
3. First checkpoint review
4. Adjust steering docs based on learnings

### Medium-term (This Month)
1. Complete all MVP Foundation tasks
2. Have working CLI
3. Basic web UI functional
4. Ready for first user testing

### Long-term (Next Quarter)
1. v1.0 release
2. User feedback incorporation
3. Phase 2 feature planning
4. Community building

---

## Conclusion

You now have:
1. ✅ **Three comprehensive steering documents**:
   - `product.md` (19KB) - Vision, scope, roadmap
   - `design.md` (16KB) - UI/UX system, accessibility
   - `tech.md` (61KB) - Complete technical architecture, infrastructure, CLI framework, and project structure
2. ✅ Refined autonomous development process with safeguards
3. ✅ Clear decision tree with exit conditions
4. ✅ Quality gates and escalation triggers
5. ✅ Checkpoint-based human validation
6. ✅ Complete implementation guide

### Document Consolidation
The steering documents have been streamlined from 6 files to 3 comprehensive documents:
- **Consolidated into tech.md**: CLI-first architecture, project structure, database schema, development tools
- **Enhanced product.md**: Added implementation roadmap and phasing
- **Kept design.md**: Comprehensive UI/UX system unchanged

This consolidation provides:
- ✅ **Single source of truth** for technical decisions
- ✅ **Easier navigation** with fewer files to reference
- ✅ **Reduced context switching** during autonomous development
- ✅ **Complete coverage** with no information loss

The key differences from your original plan:
- **Scope Boundaries**: "Won't Do" lists prevent feature bloat
- **Quality Gates**: Automated checks prevent technical debt
- **Checkpoints**: Regular human validation prevents drift
- **Completion Criteria**: Explicit "done" definition prevents infinite loops
- **Escalation**: Clear triggers for when AI should stop and ask for help
- **Comprehensive Documentation**: Single tech.md contains all architecture, infrastructure, and CLI automation details

**Start small, validate often, iterate rapidly.** Good luck! 🚀
