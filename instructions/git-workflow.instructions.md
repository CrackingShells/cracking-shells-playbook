# Git Workflow Guidelines

## Core Principles

### 1. Single Logical Change Per Commit

Each commit addresses exactly one logical change with clear rationale, enabling precise rollback capabilities and traceable development history.

### 2. Conventional Commit Format

Standardized commit messages for automated tooling integration and clear communication.

### 3. Development Narrative

Commit history tells a coherent story of development progress and decision-making.

#### Scope Consistency

For a commit sequence to form a meaningful narrative, scopes must be semantically consistent:

**Good Narrative Flow**:
```plaintext
feat(kiro): add MCP server discovery
feat(kiro): implement configuration validation
fix(kiro): handle connection errors
docs(kiro): document configuration options
```

**Bad Narrative Flow**:
```plaintext
feat(kiro): add MCP server discovery
feat(config): update settings format
fix(connection): handle timeout
chore: update dependencies
```

**Key Principles**:
- Related commits should share the same scope
- Scope changes indicate a shift in focus
- Inconsistent scopes break the development story
- Use consistent naming across related commits

---

## Conventional Commit Format

### Structure

```plaintext
<type>(scope): <description>

[optional body]

[optional footer(s)]
```

**Scope Requirement**: The scope MUST explicitly identify the topic/subject of the commit. This is critical for traceability and automated tooling.

The scope answers: "What is this commit about?" It should identify:
- A specific feature (e.g., `kiro`, `codex`)
- A component/module (e.g., `auth`, `UserService`)
- A concept (e.g., `rate-limiting`, `unicode-handling`)
- A file/module (e.g., `config.yml`, `main.py`)

**Good scopes**: `kiro`, `codex`, `auth`, `UserService`, `config`
**Bad scopes**: `core`, `refactor`, `misc`, `backend` (too generic, no traceability)

This facilitates:
- Automated changelog generation
- Versioning tools
- Commit traceability
- Development narrative consistency
=======

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

### Examples

```plaintext
# Feature implementation with explicit scopes
feat(kiro): add MCP server configuration support
feat(codex): implement API client integration
feat(auth): add OAuth2 authentication flow
feat(dashboard): implement real-time metrics visualization

# Bug fixes with specific topics
fix(codex): handle rate limiting errors
fix(auth): resolve token refresh race condition
fix(api): prevent duplicate request handling

# Documentation with clear subjects
docs(config): update configuration reference
docs(auth): document OAuth2 flow

# Testing with component focus
test(auth): add OAuth2 flow validation
test(api): add integration test coverage

# Refactoring with component identification
refactor(UserService): extract validation logic
refactor(utils): clean up helper functions
```

**Scope Analysis**:
- `feat(kiro)`: Explicit feature name
- `fix(codex)`: Specific component with issue
- `test(auth)`: Clear test subject
- `refactor(UserService)`: Component being refactored

---

## Commit Strategy Planning

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
- Identify **explicit topic** for scope and type for each change
- Plan body content for complex changes
- **Review commitlint configuration** to ensure compliance with project rules

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

## Repository Hygiene

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

## Scope Naming: Explicit Topic Identification

### Core Principle

The scope must explicitly identify the **topic/subject** of the commit. This creates traceability and enables meaningful commit history analysis.

### Scope Selection Guidelines

**Question to Answer**: "What is this commit primarily about?"

**Good Scope Patterns**:
- **Feature names**: `kiro`, `codex`, `auth`, `dashboard`
- **Component/module names**: `UserService`, `api-client`, `database`
- **Concepts**: `rate-limiting`, `unicode-handling`, `error-recovery`
- **Files/modules**: `config.yml`, `main.py`, `utils`

**Bad Scope Patterns**:
- `core` - too generic, no traceability
- `refactor` - describes action, not subject
- `misc` - provides zero information
- `backend` - architectural location, not topic

### Scope Consistency in Development Narrative

Commits about the same topic should use the same scope to create a cohesive development story:

```plaintext
# Good: Consistent scope creates traceable narrative
feat(kiro): add MCP server discovery
feat(kiro): implement configuration validation
feat(kiro): add error handling
fix(kiro): handle connection timeout

# Bad: Inconsistent scopes break narrative
feat(kiro): add MCP server discovery
feat(config): update settings format
fix(connection): handle timeout
```

### Scope Discovery Process

1. Identify the primary subject affected by this change
2. Use names from:
   - Requirements/documentation
   - Existing codebase components
   - Project-specific naming conventions
3. Check commitlint configuration for allowed scopes
4. Use the most specific scope possible

### Examples

```plaintext
# Working on MCP server support for Kiro
feat(kiro): add MCP server discovery protocol
feat(kiro): implement configuration validation
fix(kiro): handle connection errors gracefully

# Fixing authentication in Codex
fix(codex): handle expired access tokens
feat(codex): add token refresh mechanism

# Refactoring user service
refactor(UserService): extract validation logic
refactor(UserService): improve error handling
```

## Commitlint Integration

### Configuration File Requirements

**Mandatory**: All projects MUST include a commitlint configuration file. Common formats:
- `commitlintrc.json`
- `.commitlintrc.js`
- `.commitlintrc.yaml`

**Agent Responsibilities**:
1. **Locate**: Find and read the commitlint configuration file in the project root
2. **Comply**: Follow all rules defined in the configuration
3. **Validate**: Run `npx commitlint --edit` before finalizing commit messages
4. **Report**: If configuration is missing or incomplete, create/improve it

### Pre-Commit Hook Setup

**Required**: Projects must enforce commitlint via pre-commit hooks.

**Recommended Setup**:
```bash
# Install husky and commitlint
npx husky-init && npm install
npm install --save-dev @commitlint/cli @commitlint/config-conventional

# Create commitlint configuration
echo "module.exports = { extends: ['@commitlint/config-conventional'] };"> commitlint.config.js

# Add pre-commit hook
npx husky add .husky/commit-msg 'npx --no -- commitlint --edit $1'
```

### Common Commitlint Rules

Projects typically enforce:
- **Type requirements**: Only approved types (feat, fix, docs, etc.)
- **Scope requirements**: Scopes must match feature/module names
- **Subject length**: 50-72 characters maximum
- **Subject case**: Lowercase, start with verb
- **Body requirements**: Proper indentation and line breaks
- **Footer format**: `Fixes #123` or `Closes #456`

**Example commitlintrc.json**:
```json
{
  "extends": ["@commitlint/config-conventional"],
  "rules": {
    "type-enum": [2, "always", ["feat", "fix", "docs", "refactor", "test", "chore", "ci", "perf", "style"]],
    "scope-enum": [2, "always", ["kiro", "codex", "auth", "dashboard", "api", "cli"]],
    "subject-case": [2, "always", "lower-case"],
    "subject-max-length": [2, "always", 72]
  }
}
```

### Validation Workflow

1. **Before committing**:
   ```bash
   npx commitlint --edit $1
   ```

2. **In CI/CD pipeline**:
   ```yaml
   # Example GitHub Actions step
   - name: Validate commit messages
     run: npx commitlint --from=HEAD~1 --to=HEAD
   ```

3. **Local development**:
   - Install commitlint globally: `npm install -g @commitlint/cli`
   - Validate current commit: `commitlint --edit`
   - Validate commit history: `commitlint --from=HEAD~5`

## Commit Message Best Practices

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

## Quality Assurance

### Pre-Commit Checklist

- [ ] Single logical change
- [ ] Clear, descriptive commit message with **explicit topic in scope**
- [ ] Conventional commit format
- [ ] **Commitlint validation passed** (run `npx commitlint --edit`)
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

## Agent Optimization

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

**Cross-References**:
- See `git-workflow-debugging.instructions.md` for debugging workflow
- See `git-workflow-milestone.instructions.md` for milestone-based development
- See `roadmap-generation.instructions.md` for milestone/task structure
- See `testing.instructions.md` for test requirements

---

## Success Metrics

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
