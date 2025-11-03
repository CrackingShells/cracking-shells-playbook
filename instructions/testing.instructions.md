# CrackingShells Definitive Testing Standard

## Executive Summary

This standard synthesizes comprehensive analysis findings with proven organizational practices to create a practical, modern testing architecture that solves real pain points: better CLI control, readable file structure, readable test output, and industry alignment.

**Key Decisions:**

- **Wobble as standard testing framework** for all CrackingShells repositories
- **Universal hierarchical structure** with industry-standard `test_*.py` naming
- **Three-tier categorization** (development/regression/integration) with feature test migration
- **Modern CLI interface** with file output and enhanced reporting capabilities
- **Enhanced output formatting** to reduce cognitive overload
- **Practical flexibility** balanced with clear requirements

## 1. Directory Structure Standard

### 1.1 Universal Hierarchical Structure

**All repositories must use this structure:**

```plaintext
tests/
├── __init__.py
├── test_decorators.py
├── test_data_utils.py
├── test_output_formatter.py
├── development/
│   ├── __init__.py
│   └── test_*.py
├── regression/
│   ├── __init__.py
│   └── test_*.py
├── integration/
│   ├── __init__.py
│   └── test_*.py
└── test_data/
    ├── configs/
    ├── responses/
    └── events/
```

**Rationale:** Hierarchical structure improves readability and organization for growing test suites while using industry-standard file naming within categories.

### 1.2 File Naming Convention

**Standard naming pattern:**

- All test files: `test_<component>.py`
- Examples:
  - `tests/regression/test_openai_provider.py`
  - `tests/integration/test_command_system.py`
  - `tests/development/test_new_feature.py`

**Benefits:** Industry-standard naming improves IDE support and developer familiarity while directory structure provides clear categorization.

## 2. Test Categorization System

### 2.1 Three-Tier System

**Development Tests** - Temporary validation during development:

- Purpose: Drive development of new features and validate work-in-progress
- Lifecycle: Remove after feature completion or convert to regression tests
- Location: `tests/development/`
- Decorator: `@development_test(phase=1)`

**Regression Tests** - Permanent functionality validation:

- Purpose: Prevent breaking changes to existing functionality
- Lifecycle: Maintain indefinitely
- Location: `tests/regression/`
- Decorator: `@regression_test`

**Integration Tests** - Component interaction validation:

- Purpose: Validate component interactions and end-to-end workflows
- Lifecycle: Maintain indefinitely
- Location: `tests/integration/`
- Decorator: `@integration_test(scope="component|service|end_to_end")`



### 2.2 Required Decorators

**Core Categorization:**

```python
@regression_test
def test_existing_functionality(self):
    pass

@integration_test(scope="component")
def test_component_interaction(self):
    pass

@integration_test(scope="service")
def test_external_service_integration(self):
    pass

@integration_test(scope="end_to_end")
def test_complete_workflow(self):
    pass

@development_test(phase=1)
def test_new_feature_development(self):
    pass
```

**Conditional Decorators:**

```python
@slow_test
def test_performance_intensive_operation(self):
    pass

@requires_api_key
def test_external_api_integration(self):
    pass

@requires_external_service("openai")
def test_openai_provider(self):
    pass

@skip_ci
def test_local_environment_only(self):
    pass
```

### 2.3 What NOT to Test

**Principle**: Test your implementation, not existing functionality or standard library behavior.

**Do NOT test**:

- ❌ **Standard library behavior** (trust Python stdlib)
  - Example: Testing that `shlex.split()` correctly parses quoted strings
  - Instead: ✅ Test that your code correctly uses `shlex.split()` for argument parsing

- ❌ **Framework behavior** (trust Pydantic, pytest, Wobble, etc.)
  - Example: Testing that Pydantic validates required fields
  - Instead: ✅ Test that your Pydantic model has the correct field definitions

- ❌ **Existing functionality** (only test changes)
  - Example: Testing that existing `get_config()` works when you're adding `set_config()`
  - Instead: ✅ Test that `set_config()` correctly updates configuration

- ❌ **Third-party library behavior**
  - Example: Testing that `requests.get()` makes HTTP requests
  - Instead: ✅ Test that your code handles `requests.get()` responses correctly

**Concrete Examples**:

| ❌ Bad Test (Testing Existing Functionality) | ✅ Good Test (Testing Your Implementation) |
|----------------------------------------------|---------------------------------------------|
| Test that `json.dumps()` serializes dicts | Test that your `export_config()` uses `json.dumps()` correctly |
| Test that Pydantic validates types | Test that your model has correct field types and validators |
| Test that pytest fixtures work | Test that your code uses fixtures correctly |
| Test that `os.path.exists()` checks files | Test that your code handles missing files gracefully |

**Impact**: Following this principle typically reduces initial test definitions by 30-50% while maintaining the same effective coverage.

### 2.4 Test Definition Self-Review Checklist

Before submitting test definitions, apply this checklist to each test:

- [ ] **Implementation Focus**: Can I clearly state what code change I'm testing?
- [ ] **Scope Clarity**: Is this testing our implementation or existing functionality?
- [ ] **Failure Criterion**: Would this test fail if I removed the feature code?
- [ ] **Uniqueness**: Is there an existing test that already covers this scenario?
- [ ] **Trust Boundaries**: Am I testing standard library/framework behavior (which we should trust)?
- [ ] **Consolidation**: Can this test be consolidated with another test without losing coverage?
- [ ] **Value Addition**: Does this test add unique value to the test suite?

**Expected Impact**: Reduce unnecessary tests in initial definitions by 30-40%.

### 2.5 Test Organization and Grouping

**Functional Grouping** (recommended for test documentation):

Instead of arbitrary letter-based categories (A, B, C), use functional grouping that describes what is being tested:

```markdown
## Core Functionality Tests
- Server existence detection
- Partial update validation
- Configuration merging

## Field Handling Tests
- Field preservation during updates
- Command/URL switching behavior
- Header management

## Integration & Regression Tests
- End-to-end workflows
- Backward compatibility
- Error handling
```

**Benefits**:
- Clearer purpose for each group
- Easier to understand test coverage
- Simpler naming (no confusing letter schemes)
- Better organization without over-categorization

**Guideline**: Use 3-4 functional groups maximum for clarity.

### 2.6 Test Necessity Evaluation Criteria

**Before writing each test, ask**:
1. **Ownership**: Does this test our implementation or standard library/framework behavior?
2. **Uniqueness**: Does this test add unique value not covered by other tests?
3. **Consolidation**: Can this be combined with another test without losing coverage?
4. **Critical Path**: Is this testing a critical path for our feature?

**Remove tests that**:
- Validate standard library behavior (trust Python stdlib: argparse, unittest, json, pathlib, etc.)
- Validate well-established framework behavior (trust Flask, Django, FastAPI, pytest, etc.)
- Duplicate coverage of other tests without adding new insights
- Test implementation details rather than observable behavior
- Add complexity without proportional value

**Trust Standard Libraries and Frameworks**: Python's standard library and well-established frameworks are thoroughly tested by their maintainers. Focus your tests on the logic you own and the integration points between your code and these dependencies.

**Example Principle**:
- ❌ AVOID: Testing that a framework's built-in feature works as documented
- ✅ GOOD: Testing that your code correctly uses the framework's feature
- ✅ GOOD: Testing your custom logic, error handling, and business rules

## 3. Standard Testing Framework

### 3.1 Wobble as Organizational Standard

**All CrackingShells repositories must use Wobble as the standard testing framework.**

**Installation:**
```bash
# Install from repository
pip install git+https://github.com/CrackingShells/Wobble.git

# For development/editable installation
git clone https://github.com/CrackingShells/Wobble.git
cd Wobble
pip install -e .
```

**Basic Usage:**
```bash
# Run all tests
wobble

# Run specific categories
wobble --category regression
wobble --category integration
wobble --category development

# File output for CI/CD
wobble --log-file ci_results.json --log-file-format json
```

#### Essential Wobble Flags for Development

**File Output for Stakeholder Review** (recommended for all development work):
```bash
wobble --log-file test_execution_v0.txt --log-verbosity 3
```

**Key Flags**:
- `--log-file <filename>`: Save test results to file (auto-detects format from extension: .txt or .json)
- `--log-verbosity <level>`: Output detail level
  - Level 1: Minimal (CI/CD pipelines)
  - Level 2: Balanced (local development)
  - Level 3: Complete (stakeholder review, debugging)
- `--pattern <glob>`: Select specific test files (e.g., `test_cli_*.py`)

**Version Log Files** to track debugging iterations:
```bash
# Initial run
wobble --log-file test_execution_v0.txt --log-verbosity 3

# After fixes
wobble --log-file test_execution_v1.txt --log-verbosity 3
```

**Discover More Options**:
```bash
wobble --help
```

Use `--help` to explore additional flags for category filtering, output formats, and advanced features.

### 3.2 Migration Guidelines

**For Existing Repositories:**

1. **Install Wobble**: Add wobble dependency to `pyproject.toml`
2. **Update Test Structure**: Migrate to hierarchical structure if needed
3. **Add Decorators**: Apply required categorization decorators
4. **Update CI/CD**: Replace existing test runners with wobble commands
5. **Remove Legacy**: Remove old test runner scripts and configurations

**Migration Example:**
```bash
# Before (legacy)
python -m pytest tests/

# After (wobble standard)
wobble --category regression --log-file ci_results.json
```

### 3.3 Wobble Features and Benefits

**Enhanced Test Execution:**
- Automatic test discovery with hierarchical and decorator support
- Category-based test filtering and execution
- Performance tracking with timing information
- Enhanced error reporting with actionable messages

**File Output Capabilities:**
- JSON and text format support with auto-detection
- Configurable verbosity levels (1=Basic, 2=Detailed, 3=Complete)
- Concurrent file writing without blocking test execution
- CI/CD integration with structured result files

**Advanced Features:**
- Cross-platform Unicode symbol support
- Threaded file I/O for performance
- Command replay and reconstruction
- Comprehensive metadata tracking

### 3.4 CI/CD Integration Standards

**Required CI/CD Configuration:**
```yaml
# GitHub Actions example
- name: Run Tests with Wobble
  run: |
    wobble --category regression --log-file ci_results.json --log-verbosity 3
    wobble --category integration --log-file integration_results.json
```

**File Output Requirements:**
- All CI/CD pipelines must use `--log-file` for result archiving
- Use JSON format for programmatic parsing: `--log-file-format json`
- Include high verbosity for debugging: `--log-verbosity 3`
- Archive result files as CI/CD artifacts

## 4. Enhanced Developer Experience

### 4.1 Output Formatting

**Retain enhanced output formatting** (`test_output_formatter.py`) with these features:

- Status icons (✅❌⏭️💥) with color coding
- Category headers for clear test organization
- Detailed timing and metadata display
- Failure details with file locations and error context
- Adaptive terminal width handling

**Example Output:**

```plaintext
=== REGRESSION TESTS ===
test_openai_provider_initialization ..................... ✅ PASS (0.12s) [regression]
test_authentication_with_valid_credentials .............. ✅ PASS (0.08s) [regression]

=== INTEGRATION TESTS ===
test_end_to_end_chat_workflow
  Class: TestChatWorkflow
  Tags: @integration_test @requires_api_key @scope_end_to_end
  Duration: 4.23s ✅ PASS

=== TEST EXECUTION SUMMARY ===
Total Tests: 15   Passed: 14   Failed: 1   Skipped: 0
Total Duration: 12.45s   Average: 0.83s
```

### 4.2 Test Independence Guidelines

**Resource Management (from original guidelines):**

- Use context managers for automatic resource cleanup
- Create unique identifiers for temporary resources (timestamps, UUIDs)
- Clean up files, directories, and background processes in tearDown methods
- Mock time-sensitive operations to avoid race conditions

**Implementation Example:**

```python
class TestUserAuthentication(unittest.TestCase):
    def setUp(self):
        self.temp_dir = tempfile.mkdtemp(prefix=f"test_{uuid.uuid4().hex[:8]}_")
        self.mock_time = patch('time.time', return_value=1234567890)
        self.mock_time.start()
    
    def tearDown(self):
        shutil.rmtree(self.temp_dir, ignore_errors=True)
        self.mock_time.stop()
```

### 4.3 Mock vs Real Integration Strategy

**Decision Framework (from original guidelines):**

- **Unit tests**: Mock external dependencies (API calls, file system, network)
- **Integration tests**: Use real implementations when testing component interactions
- **Simple rule**: If testing single class/function logic → mock. If testing how components work together → real.

**Implementation Guidance:**

```python
# Unit test - mock external dependencies
@regression_test
def test_api_client_handles_timeout(self):
    with patch('requests.post') as mock_post:
        mock_post.side_effect = requests.Timeout()
        # Test timeout handling logic

# Integration test - use real implementations
@integration_test(scope="service")
def test_openai_provider_with_real_api(self):
    # Test actual API integration with real OpenAI service
```

### 4.4 Mock Patching Best Practices

**Critical Rule**: Always patch where a function is **used**, not where it's **defined**.

#### Import Style Determines Patch Target

**For `from X import Y` imports**:
```python
# In module.py
from some_library import some_function

def my_function():
    return some_function()

# In test file - patch at point of use
with patch('module.some_function', return_value='mocked'):
    result = my_function()  # Returns 'mocked'
```

**For `import X` imports**:
```python
# In module.py
import some_library

def my_function():
    return some_library.some_function()

# In test file - patch at definition
with patch('some_library.some_function', return_value='mocked'):
    result = my_function()  # Returns 'mocked'
```

#### Why This Matters

When Python executes `from X import Y`, it creates a **local reference** to `Y` in the importing module's namespace. Patching `X.Y` after import doesn't affect this local reference. You must patch the reference where it's actually used.

**This pattern applies universally** to any Python module, function, class, or constant being mocked, not just the specific libraries shown in examples.

#### Debugging Mock Issues

If your mock isn't being applied:
1. Check the import style in the module being tested
2. Verify you're patching `module_using_it.function` not `original_module.function`
3. Use `print()` statements to confirm which code path is executing
4. Consider using `patch.object()` for more explicit control

## 6. Test Data Management

### 6.1 Structured Test Data

**Example Directory Structure:**

```plaintext
tests/test_data/
├── configs/          # Test configuration files
├── responses/        # Mock API/tool responses
├── events/           # Sample event payloads
└── fixtures/         # Database and object fixtures
```

### 6.2 Data Access Utilities

**Standardized helper functions:**

```python
from tests.test_data_utils import load_test_config, load_mock_response

# Load test configuration
config = load_test_config("test_settings")

# Load mock response data
response = load_mock_response("api_success_responses")
```

## 7. Maintenance and Quality Standards

### 7.1 Test Maintenance Procedures

**Development Test Cleanup (from original guidelines):**

- Review development tests monthly
- Remove tests for completed features before merging
- Convert valuable development tests to regression tests

**Stale Test Detection:**

- Run automated detection for tests not modified in 90+ days
- Review flagged tests quarterly for relevance
- Update or remove outdated tests

### 7.2 Quality Requirements

**Coverage Standards:**

- Minimum coverage: 70% for all repositories
- Critical functionality: 90% coverage required
- New features: 95% coverage required

**Performance Standards:**

- Unit tests: <1 second per test
- Integration tests: <10 seconds per test
- Full CI test suite: <5 minutes execution time

### 7.3 Assertion Best Practices

**Descriptive assertions (from original guidelines):**

```python
# Good - descriptive assertion messages
self.assertEqual(result.status, "authenticated",
                f"Authentication failed: expected 'authenticated', got '{result.status}'")

# Good - assert both state and side effects
self.assertTrue(user.is_authenticated)
self.assertIn("login_success", captured_events)
```

### 7.4 Test-to-Code Ratio Guidelines

**Target Ratios** (tests : code changes):

- **Bug Fixes**: 2:1 to 3:1
  - Focus on regression test for the bug
  - Validation that existing functionality remains intact
  - Example: 6 bug fixes → Target: 12-18 tests

- **New Features**: 4:1 to 6:1
  - Comprehensive coverage of new functionality
  - Edge cases and error handling
  - Integration with existing code
  - Example: 1 medium complexity feature → Target: 8-12 tests

- **Refactoring**: 1:1 to 2:1
  - Verify behavior preservation
  - Minimal new tests (refactoring shouldn't change behavior)
  - Example: 1 refactoring → Target: 2-4 tests (if any new tests needed)

**Example Analysis**:
- 6 issues (bug fixes) → Target: 12-18 tests
- 1 feature (medium complexity) → Target: 8-12 tests
- 1 refactoring → Target: 2-4 tests (if any new tests needed)
- **Total**: 22-34 tests for this scope

**Red Flags**:
- 23 tests for 6 bug fixes (3.8:1) → Likely over-testing
- 5 tests for complex feature → Likely under-testing
- 15 tests for simple refactoring → Likely testing existing functionality

**Note**: These are guidelines, not strict rules. Adjust based on:
- Complexity of the change
- Risk level (critical vs non-critical paths)
- Existing test coverage
- Integration complexity

**Application**: Use these ratios during test definition review to validate test scope appropriateness.

---

This standard provides a practical, modern testing architecture that solves real pain points while preserving proven organizational practices and maintaining focus on developer productivity.
