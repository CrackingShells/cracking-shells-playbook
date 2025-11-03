---
applyTo: 'docs/articles/api/**/*.md, **/*.py'
description: 'API documentation generation standards using mkdocstrings and Google-style docstrings.'
---

# API Documentation Instructions

## Overview
This file defines standards for automated API documentation generation using mkdocstrings with Google-style docstrings for consistent, professional API reference documentation.

## API Documentation Generation

### Creating API Reference Files
For each Python module in your package, create a corresponding markdown file in `docs/articles/api/`:

#### Structure Pattern
```
docs/articles/api/
├── index.md                     # API overview and getting started
├── [module_name].md            # Individual module documentation
├── [subpackage]/               # Subpackage organization
│   ├── [submodule].md         # Submodule documentation
│   └── index.md               # Subpackage overview
```

#### File Content Pattern
Each API documentation file should use the mkdocstrings syntax:

```markdown
# [Module Name]

::: [package].[module_name]
```

#### Examples

**For a module `hatch/cli_hatch.py`, create `docs/articles/api/cli.md`:**
```markdown
# CLI Module

::: hatch.cli_hatch
```

**For a submodule `hatch/installers/docker_installer.py`, create `docs/articles/api/installers/docker.md`:**
```markdown
# Docker Installer

::: hatch.installers.docker_installer
```

### API Index Page
Create `docs/articles/api/index.md` with:

```markdown
# API Reference

Welcome to the [Repository Name] API Reference documentation. This section provides detailed documentation for all public APIs and modules.

## Overview

[Brief description of the package and its main components]

The API is organized into several key areas:
- **Core Modules**: Main functionality and primary interfaces
- **[Subpackage Name]**: Specialized functionality for [purpose]
- **[Additional Areas]**: Other important components

## Getting Started

To use [Package Name] programmatically, you can import the main modules:

```python
from [package] import [main_module]
from [package].[subpackage] import [SubClass]

# Basic usage example
instance = [SubClass]()
result = instance.method()
```

## Module Index

Browse the detailed API documentation for each module using the navigation on the left.

### Core Modules
- **[Module Name]**: [Brief description of module purpose]
- **[Other Module]**: [Brief description]

### [Subpackage Name]
- **[Submodule]**: [Brief description]
- **[Other Submodule]**: [Brief description]
```

### Subpackage Index Pages
For subpackages, create `docs/articles/api/[subpackage]/index.md`:

```markdown
# [Subpackage Name]

The [subpackage name] subpackage provides [description of functionality].

## Modules

- **[Module 1]**: [Brief description]
- **[Module 2]**: [Brief description]

## Common Usage Patterns

[Examples of typical usage patterns for the subpackage]

```python
from [package].[subpackage] import [Module]

# Example usage
example = [Module]()
result = example.common_method()
```
```

## Docstring Requirements

### Google-Style Docstrings
All Python code must use Google-style docstrings for proper mkdocstrings integration:

#### Function Documentation Pattern
```python
def example_function(param1: str, param2: int = 0) -> bool:
    """Brief description of the function.

    Longer description explaining the function's purpose,
    behavior, and any important details. This section can
    span multiple paragraphs if needed.

    Args:
        param1: Description of the first parameter.
        param2: Description of the second parameter with default.
            Can include additional details on multiple lines.

    Returns:
        Description of the return value and its type.

    Raises:
        ValueError: Description of when this exception is raised.
        TypeError: Description of when this exception is raised.

    Example:
        Basic usage example:

        ```python
        result = example_function("hello", 42)
        print(result)  # True
        ```

    Note:
        Additional notes about usage, performance, or behavior.
    """
    pass
```

#### Class Documentation Pattern
```python
class ExampleClass:
    """Brief description of the class.

    Longer description explaining the class's purpose and usage.
    Include information about when to use this class and how
    it fits into the broader system.

    Attributes:
        attribute1 (str): Description of the attribute.
        attribute2 (int): Description of another attribute.
        _private_attr (bool): Private attributes can be documented
            but won't appear in public API docs.

    Example:
        Basic usage:

        ```python
        instance = ExampleClass()
        instance.public_method("data")
        print(instance.attribute1)
        ```
    """

    def __init__(self, param1: str, param2: Optional[int] = None):
        """Initialize the ExampleClass instance.

        Args:
            param1: Required parameter for initialization.
            param2: Optional parameter with default behavior.
        """
        self.attribute1 = param1
        self.attribute2 = param2 or 0

    def public_method(self, data: str) -> str:
        """Process data and return result.

        Args:
            data: Input data to process.

        Returns:
            Processed result.

        Raises:
            ValueError: If data is empty or invalid.
        """
        if not data:
            raise ValueError("Data cannot be empty")
        return f"processed: {data}"
```

#### Module-Level Documentation
```python
"""Module for handling example functionality.

This module provides classes and functions for [specific purpose].
It integrates with [other components] to enable [functionality].

Typical usage example:

    ```python
    from package import example_module
    
    handler = example_module.ExampleClass()
    result = handler.process("data")
    ```

Classes:
    ExampleClass: Main class for handling [functionality].
    HelperClass: Utility class for [specific tasks].

Functions:
    utility_function: Standalone function for [purpose].
    
Constants:
    DEFAULT_VALUE: Default configuration value.
"""

from typing import Optional

DEFAULT_VALUE = "default"
```

### Docstring Quality Standards

#### Required Elements
- **Brief description**: One-line summary of purpose
- **Detailed description**: Longer explanation when needed
- **Args section**: All parameters with types and descriptions
- **Returns section**: Return value type and description
- **Raises section**: All possible exceptions
- **Example section**: Practical usage examples

#### Best Practices
- Use present tense ("Returns the result" not "Will return the result")
- Be specific about parameter types and constraints
- Include practical, runnable examples
- Document edge cases and special behaviors
- Keep descriptions concise but complete

#### Type Hints Integration
- Use type hints in function signatures
- Docstring types should match type hints
- mkdocstrings will automatically display type information
- Focus docstring descriptions on behavior, not just types

## Navigation Integration

### MkDocs Navigation Structure
API documentation should be integrated into the main navigation:

```yaml
nav:
  - API Reference:
      - Overview: articles/api/index.md
      - Core Modules:
          - CLI: articles/api/cli.md
          - Environment Manager: articles/api/environment_manager.md
          - Package Loader: articles/api/package_loader.md
      - Installers:
          - Overview: articles/api/installers/index.md
          - Base Installer: articles/api/installers/base.md
          - Docker Installer: articles/api/installers/docker.md
          - Python Installer: articles/api/installers/python.md
```

### Logical Organization
- Group related modules together
- Use submenus for subpackages
- Maintain consistent naming patterns
- Order by importance and typical usage flow

## Code Quality Requirements

### Public API Standards
- **All public modules** must have corresponding API documentation files
- **All public classes and functions** must have complete Google-style docstrings
- **All parameters and return values** must be documented
- **All exceptions** that can be raised must be documented

### Documentation Coverage
- Public APIs require 100% docstring coverage
- Private methods may have docstrings but won't appear in generated docs
- Internal implementation details should be documented in code comments, not docstrings

### Example Quality
- All examples in docstrings must be runnable
- Examples should demonstrate realistic usage patterns
- Include both basic and advanced usage examples where appropriate
- Test examples as part of documentation validation

## Automated Generation Process

### mkdocstrings Configuration
The following configuration in `mkdocs.yml` controls API documentation generation:

```yaml
plugins:
  - mkdocstrings:
      default_handler: python
      handlers:
        python:
          options:
            docstring_style: google
            show_source: true
            show_root_heading: true
            show_object_full_path: false
            show_category_heading: true
            show_labels: true
            show_symbol_type_heading: true
            show_symbol_type_toc: true
```

#### Configuration Details
- **docstring_style: google**: Processes Google-style docstrings
- **show_source: true**: Includes source code links
- **show_root_heading: true**: Shows module name as main heading
- **show_object_full_path: false**: Uses short names for readability
- **show_category_heading: true**: Groups by classes/functions/etc.
- **show_labels: true**: Shows property/method labels
- **show_symbol_type_heading: true**: Shows type information
- **show_symbol_type_toc: true**: Includes types in table of contents

### Build Process
1. **mkdocstrings scans** API documentation files for `:::` syntax
2. **Python modules are imported** and introspected
3. **Docstrings are parsed** according to Google style
4. **HTML documentation is generated** with navigation and cross-references
5. **Source code links** are created for easy reference

## Quality Assurance

### Documentation Validation
- Verify all public modules have API documentation files
- Check that all API files contain valid mkdocstrings references
- Ensure navigation structure matches actual module organization
- Test that all examples in docstrings work correctly

### Build Testing
- Run `mkdocs serve` to test local generation
- Verify API sections render correctly
- Check that cross-references and links work
- Ensure search functionality includes API content

### Continuous Validation
- Include documentation checks in CI/CD pipelines
- Validate docstring completeness for new code
- Test example code in docstrings
- Monitor for broken API documentation links

## Troubleshooting

### Common Issues

**Module Import Errors**:
- Ensure Python path includes package directory
- Check that all dependencies are installed
- Verify module names match exactly in `:::` references

**Missing Documentation**:
- Check that functions/classes have docstrings
- Verify Google-style docstring format
- Ensure public APIs are not excluded by naming conventions

**Formatting Problems**:
- Validate docstring indentation and structure
- Check for special characters that might break parsing
- Ensure code blocks in examples are properly formatted

**Navigation Issues**:
- Verify file paths in navigation match actual files
- Check that API documentation files exist
- Ensure subpackage index pages are properly linked

## Maintenance

### Regular Tasks
- Review API documentation for completeness
- Update examples when APIs change
- Add documentation for new public modules
- Remove documentation for deprecated APIs

### Automation Opportunities
- Script to generate API documentation files from module structure
- Automated checking of docstring completeness
- Integration with linting tools to enforce documentation standards
- Automated testing of docstring examples

## Related Guidelines
- See [MkDocs Setup](./documentation-mkdocs-setup.instructions.md) for technical configuration
- See [Documentation Structure](./documentation-structure.instructions.md) for organization
- See [Documentation Style Guide](./documentation-style-guide.instructions.md) for writing standards
- See [Python Docstrings](./python_docstrings.instructions.md) for detailed docstring standards