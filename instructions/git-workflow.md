# GIT WORKFLOW GUIDELINES

## CORE PRINCIPLES

### 1. Single Logical Change Per Commit

Each commit addresses exactly one logical change with clear rationale, enabling precise rollback capabilities and traceable development history.

### 2. Conventional Commit Format

Standardized commit messages for automated tooling integration and clear communication.

### 3. Development Narrative

Commit history tells a coherent story of development progress and decision-making.

---

## CONVENTIONAL COMMIT FORMAT

### Structure

```plaintext
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Leverage scopes of the commit types to facilitate categorization. This is typically useful with automated changelog generation and versioning tools. For example:

- Indicate whether a feature is for CLI, API, or core functionality
- whether a fix is for a feature in development, the tests, core feature
- on what the refactoring applies
- And so on...

### Types

- **feat**: New features (triggers minor version bump)
- **fix**: Bug fixes (triggers patch version bump)
- **docs**: Documentation changes
- **refactor**: Code refactoring without functional changes
- **test**: Adding or updating tests
- **chore**: Maintenance tasks, dependency updates
- **ci**: Changes to CI/CD configuration
- **perf**: Performance improvements
- **style**: Code style changes (formatting, etc.)

### Examples from Wobble Implementation

```plaintext
# Feature implementation
feat: add threaded file writer with queue-based operations
feat(cli): implement file output arguments with validation
feat(architecture): add observer pattern for multi-destination output

# Bug fixes
fix: resolve threading deadlock in file writer shutdown
fix(test): handle unittest _ErrorHolder objects gracefully
fix(feature-dev): prevent duplicate output in file logging

# Documentation
docs(user-cli): update CLI reference with file output options
docs(dev-architecture): add architecture overview for threading system
docs: enhance error message documentation

# Testing
test: add comprehensive threading and file I/O coverage
test: validate unicode symbol cross-platform compatibility
test: ensure proper resource cleanup in all scenarios

# Refactoring
refactor(architecture): extract observer pattern for output coordination
refactor: simplify unicode symbol handling logic
refactor: remove sys.path.insert statements from test files
```

---

## COMMIT STRATEGY PLANNING

### Before Starting Work

**1. Logical Change Identification**:

- Break down work into single-purpose changes
- Identify dependencies between changes
- Plan commit sequence for logical progression

**2. Rollback Strategy**:

- Ensure each commit can be safely reverted
- Avoid commits that break functionality
- Plan for partial rollback scenarios

**3. Message Planning**:

- Prepare clear, descriptive commit messages
- Identify scope and type for each change
- Plan body content for complex changes

### During Development

**1. Incremental Commits**:

- Commit frequently with small, focused changes
- Validate functionality after each commit
- Maintain working state at every commit

**2. Commit Message Quality**:

- Write clear, concise descriptions
- Explain the "why" not just the "what"
- Include context for future developers

**3. Scope Management**:

- Keep changes within defined scope
- Avoid mixing unrelated changes
- Create separate commits for different concerns

---

## REPOSITORY HYGIENE

### Branch Management

**Branch Naming**:

```plaintext
# Feature branches
feature/file-output-system
feature/threading-architecture

# Bug fix branches
fix/errorholder-compatibility
fix/duplicate-output-bug

# Documentation branches
docs/cli-reference-update
docs/architecture-overview
```

**Branch Strategy**:

- Create feature branches for significant changes
- Use descriptive branch names
- Keep branches focused on single features
- Regular rebase to maintain clean history

### Special Case: Debugging Workflow

When debugging issues, use a dedicated debugging branch to encourage frequent commits without polluting the main development history.

**Workflow**:

1. **Create debugging branch from current location**:
   ```bash
   git checkout -b debugging/<issue-description>
   # Example: debugging/memory-leak-in-profiler
   ```

2. **Commit frequently during debugging**:
   - Commit every hypothesis test
   - Commit every diagnostic change
   - Commit every attempted fix
   - Use descriptive messages: `debug: test hypothesis X`, `debug: add logging for Y`

3. **When bug is fixed**:

   **Option A: Clean commit on original branch** (recommended for simple fixes)
   ```bash
   # Identify the fix
   git log  # Review debugging commits to understand the fix

   # Switch back to original branch
   git checkout <original-branch>

   # Make ONE clean commit with the fix
   git add <fixed-files>
   git commit -m "fix: resolve memory leak in profiler shutdown

   Root cause: ThreadedFileWriter not properly joining threads
   Solution: Add explicit thread.join() with timeout in shutdown()

   Fixes #123"
   ```

   **Option B: Sequence of clean commits** (for complex fixes)
   ```bash
   # Switch back to original branch
   git checkout <original-branch>

   # Make a sequence of logical commits
   git commit -m "refactor: extract thread cleanup logic"
   git commit -m "fix: add thread join with timeout"
   git commit -m "test: add thread cleanup validation"
   ```

4. **Delete debugging branch**:
   ```bash
   git branch -D debugging/<issue-description>
   ```

**Benefits**:
- Encourages thorough debugging with frequent commits
- Preserves debugging history for learning (before deletion)
- Keeps main development history clean and logical
- Facilitates root cause analysis documentation

**Debugging Commit Format**:
```plaintext
debug: <what you're testing/trying>

# Examples:
debug: test hypothesis that memory leak is in file writer
debug: add logging to track thread lifecycle
debug: try alternative shutdown sequence
```

**Final Fix Commit Format**:
```plaintext
fix: <what was fixed>

Root cause: <why the bug occurred>
Solution: <how it was fixed>

Fixes #<issue-number>
```

**Cross-Reference**: See `work-ethics.instructions.md` for root cause analysis guidelines.

### Milestone-Based Development Workflow

For projects with formal roadmaps and milestone tracking, use hierarchical branching to organize work by phases, milestones, and tasks.

**Branch Hierarchy**:

```plaintext
main (production releases only)
  └── dev (development integration branch)
      ├── milestone/1.1-short-description
      │   ├── task/1.1.1-short-description
      │   ├── task/1.1.2-short-description
      │   └── task/1.1.3-short-description
      ├── milestone/1.2-short-description
      │   └── task/1.2.1-short-description
      └── milestone/2.1-short-description
          └── task/2.1.1-short-description
```

**Workflow Rules**:

1. **All work from `dev` branch** (not `main`)
   - `main` is production-only, receives merges only at major releases
   - `dev` is the integration branch for all development work

2. **Milestone branches from `dev`**
   - Branch naming: `milestone/<milestone-id>-<short-description>`
   - Example: `milestone/2.1-thread-safe-architecture`
   - Created when milestone work begins
   - Deleted after merge back to `dev`

3. **Task branches from milestone branches**
   - Branch naming: `task/<task-id>-<short-description>`
   - Example: `task/2.1.1-design-architecture`
   - Created when task work begins
   - Deleted after merge back to milestone branch

4. **Merge Hierarchy**:
   - Task branches → Milestone branch (when task complete)
   - Milestone branch → `dev` (when ALL milestone tasks complete)
   - `dev` → `main` (when ready for production release)

5. **Merge Criteria**:
   - **Task → Milestone**: Task success gates met, task tests pass
   - **Milestone → dev**: All milestone tasks complete, all milestone tests pass, no regressions
   - **dev → main**: ALL tests pass (regression, unit, integration, performance), ready for release

6. **Conventional Commits**: Follow organization's conventional commit format to enable automated semantic versioning

**Example Workflow**:

```bash
# Start milestone work
git checkout dev
git checkout -b milestone/1.1-core-functionality

# Start task work
git checkout -b task/1.1.1-implement-parser

# Work on task with conventional commits
git commit -m "feat(parser): add basic parsing logic"
git commit -m "test(parser): add parser validation tests"

# Complete task - merge to milestone
git checkout milestone/1.1-core-functionality
git merge task/1.1.1-implement-parser
git branch -d task/1.1.1-implement-parser

# Complete all milestone tasks, merge to dev
git checkout dev
git merge milestone/1.1-core-functionality
git branch -d milestone/1.1-core-functionality

# When ready for release, merge to main
git checkout main
git merge dev
git tag v1.1.0
```

**Benefits**:
- Clear organization of complex multi-phase projects
- Easy tracking of milestone progress
- Facilitates parallel development on different milestones
- Git history reflects roadmap structure
- Enables automated changelog generation per milestone

**Cross-References**:
- See `roadmap-generation.instructions.md` for milestone/task structure
- See `testing.instructions.md` for test requirements at each merge level

### Merge Strategy

**Merge Commit Messages**:

```plaintext
# Good merge messages
Merge pull request #123 from feature/file-output-system

feat: implement comprehensive file output system

- Add threaded file writer with queue-based operations
- Implement observer pattern for multi-destination output
- Add CLI arguments for file output configuration
- Include comprehensive test coverage and documentation

Resolves #456, #789
```

**Quality Checks**:

- All tests pass before merge
- Documentation updated for changes
- Code review completed
- No merge conflicts

### History Preservation

**Rebase vs Merge**:

- Use rebase for feature branches to maintain linear history
- Use merge commits for significant feature integration
- Preserve context through detailed merge messages
- Avoid force-pushing to shared branches

**Tag Creation**:

Handled by CI/CD pipelines based on commit content and branch rules.

---

## COMMIT MESSAGE BEST PRACTICES

### Description Guidelines

**Clear and Concise**:

```plaintext
# Good
feat: add threaded file writer with queue-based operations

# Avoid
feat: add some file stuff and fix things
```

**Imperative Mood**:

```plaintext
# Good
fix: resolve threading deadlock in file writer shutdown

# Avoid
fix: resolved threading deadlock in file writer shutdown
```

**Specific and Actionable**:

```plaintext
# Good
refactor: extract observer pattern for output coordination

# Avoid
refactor: improve code structure
```

### Body Content

**When to Include Body**:

- Complex changes requiring explanation
- Breaking changes with migration notes
- Context for future developers
- References to issues or discussions

**Body Structure**:

```plaintext
feat: implement comprehensive file output system

Add threaded file writer using queue-based operations to enable
non-blocking file I/O during test execution. Implements observer
pattern for coordinating output between console and file destinations.

Key components:
- ThreadedFileWriter: Background file operations with graceful shutdown
- OutputObserver: Multi-destination output coordination
- CLI integration: Complete argument parsing and validation

Resolves #456: File output feature request
Addresses #789: Performance concerns with blocking I/O
```

### Footer Guidelines

**Breaking Changes**:

```plaintext
feat!: change file output API interface

BREAKING CHANGE: File output configuration now uses observer pattern
instead of direct formatter calls. Update existing integrations to use
new OutputObserver interface.

Migration guide: docs/migration/file-output-v2.md
```

**Issue References**:

```plaintext
fix: handle unittest _ErrorHolder objects gracefully

Resolves #123
Fixes #456
Closes #789
```

---

## QUALITY ASSURANCE

### Pre-Commit Checklist

- [ ] Single logical change
- [ ] Clear, descriptive commit message
- [ ] Conventional commit format
- [ ] All tests pass
- [ ] No unrelated changes included
- [ ] Documentation updated if needed

### Commit Review Process

- Review commit message for clarity
- Verify change scope is appropriate
- Ensure rollback capability
- Check for conventional format compliance

### History Validation

- Commit history tells coherent story
- Each commit builds logically on previous
- No broken states in commit history
- Clear progression toward objectives

---

## AGENT OPTIMIZATION

### For AI Coding Agents

- **Context Understanding**: Focused commits provide clear context for future work
- **Change Tracking**: Conventional format enables automated analysis
- **Rollback Capability**: Single-purpose commits enable precise rollback
- **Development Narrative**: Clear history helps understand decision progression

### For Human Developers

- **Code Review**: Focused commits make review more effective
- **Debugging**: Clear history helps identify when issues were introduced
- **Knowledge Transfer**: Commit messages provide context for decisions
- **Maintenance**: Logical changes make future modifications easier

---

## SUCCESS METRICS

### Quality Indicators

- Each commit can be understood independently
- Rollback of any commit doesn't break functionality
- Commit history tells clear development story
- Automated tooling can parse commit messages

### Wobble Implementation Evidence

- ✅ 18+ focused commits with clear development narrative
- ✅ Conventional commit format for automated tooling
- ✅ Precise rollback capabilities for each change
- ✅ Clear progression from initial implementation to production

**Last Updated**: Based on Wobble implementation success (2024)
