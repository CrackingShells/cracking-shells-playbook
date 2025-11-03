---
applyTo: '**/README.md'
description: 'Documentation style guide for README files.'
---

# README Instructions

## Philosophy

A README file is the first impression and primary entry point for any repository. It must be direct, welcoming, and comprehensive enough to orient all audiences while remaining concise. The README serves as both a landing page and navigation hub to deeper documentation.

## Essential Content Requirements

### Core Information (Required)

Every README must answer the fundamental questions:

**What** - Project Identity:

- Clear, descriptive project name and tagline
- Brief description of the project's main purpose and functionality
- Primary use cases and target audience

**Why** - Value Proposition:

- Problem the project solves or need it addresses
- Key benefits and differentiators
- When and why someone would choose this project

**How** - Getting Started:

- Installation instructions with prerequisites
- Basic usage examples with working code
- Quick start guide or minimal working example
- Links to comprehensive documentation

**Who** - Community and Support:

- How to contribute and get involved
- Where to ask questions or report issues
- Maintainer information and contact methods
- Community guidelines and code of conduct references

### Status and Metadata

- **Project Status**: Current development stage (stable, beta, experimental, maintenance)
- **Version Information**: Current version and compatibility requirements
- **License**: Clear license information with link to full license text
- **Build Status**: Automated build and test status from CI/CD pipelines

## Technical Implementation Guidelines

### Badge Standards

Include relevant badges at the top of the README in this order:

````markdown
[![Build Status](https://github.com/CrackingShells/[Repository-Name]/workflows/CI/badge.svg)](https://github.com/CrackingShells/[Repository-Name]/actions)
[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Python Version](https://img.shields.io/badge/python-3.12%2B-blue.svg)](https://python.org)

#### Repository Examples

```markdown
<!-- GitHub -->
[![GitHub release](https://img.shields.io/github/v/release/crackingshells/[repository-name])](https://github.com/crackingshells/[repository-name]/releases)
[![GitHub issues](https://img.shields.io/github/issues/crackingshells/[repository-name])](https://github.com/crackingshells/[repository-name]/issues)

<!-- PyPI -->
[![PyPI version](https://img.shields.io/pypi/v/[package-name])](https://pypi.org/project/[package-name]/)
[![Python versions](https://img.shields.io/pypi/pyversions/[package-name])](https://pypi.org/project/[package-name]/)

<!-- Documentation -->
[![Documentation](https://img.shields.io/badge/docs-available-brightgreen)](https://crackingshells.github.io/[repository-name]/)

<!-- Quality -->
[![codecov](https://codecov.io/gh/crackingshells/[repository-name]/branch/main/graph/badge.svg)](https://codecov.io/gh/crackingshells/[repository-name])
[![Code style: black](https://img.shields.io/badge/code%20style-black-000000.svg)](https://github.com/psf/black)
```
````

**Badge Guidelines**:

- Only include badges that provide genuine value and are automatically updated
- Verify that linked workflows exist in `.github/workflows/` before adding build badges
- Use consistent styling and colors across organization repositories
- Remove or update badges when they become stale or irrelevant

### Installation Instructions

#### For Python Projects

Include PyPI installation instructions only if the package is published on PyPI (check the workflow), otherwise omit this section.

````markdown
## Installation

### Prerequisites
- Python 3.12 or higher
- pip package manager

### Install from PyPI
```bash
pip install [package-name]
```

### Install for Development
```bash
git clone https://github.com/CrackingShells/[Repository-Name].git
cd [Repository-Name]
pip install -e .[dev]
```
````

#### For Tool/CLI Projects

````markdown
## Installation

### Quick Install
```bash
pip install [package-name]
```

### Verify Installation
```bash
[command-name] --version
```
````

### Code Examples and Usage

#### Basic Usage Example

Always include a minimal, working example:

````markdown
## Quick Start

```python
from [package] import [main_class]

# Basic usage
instance = [main_class]()
result = instance.process("example input")
print(result)
```

For more examples, see the [User Guide](docs/articles/users/GettingStarted.md).
````

#### Code Block Standards

- Use appropriate syntax highlighting (python, bash, yaml, etc.)
- Ensure all examples are runnable and tested
- Include expected output when helpful
- Keep examples focused and minimal
- Provide links to comprehensive examples in documentation

## Documentation Integration

### Link Structure

Integrate with the organization's documentation system:

````markdown
## Documentation

📚 **[Complete Documentation](https://crackingshells.github.io/[Repository-Name]/)**

### Quick Links
- 🚀 [Getting Started](docs/articles/users/GettingStarted.md)
- 📖 [User Guide](docs/articles/users/)
- 🔧 [API Reference](docs/articles/api/)
- 👩‍💻 [Developer Guide](docs/articles/devs/)
- 🤝 [Contributing](docs/articles/devs/contribution_guidelines/)
````

### Documentation Cross-References

- **Always link to the published documentation site** as the primary reference
- **Provide direct links** to key sections users need immediately
- **Use relative paths** for links to files within the repository
- **Include context** for each link to help users choose the right resource

### Integration with MkDocs

When using MkDocs-generated documentation:

- Link to the main documentation site (ReadTheDocs or GitHub Pages)
- Reference the standardized navigation structure from [Documentation Structure Guidelines](./documentation-structure.instructions.md)
- Ensure README complements rather than duplicates comprehensive documentation

## Quality Standards

### Accessibility and Inclusivity

- Use plain language accessible to non-native speakers
- Provide clear visual hierarchy with appropriate heading levels
- Include alt text for any images or diagrams
- Use inclusive language and avoid assumptions about user background
- Consider screen reader compatibility in formatting choices

### Visual Elements

- Use consistent emoji and icons sparingly for visual navigation aids
- Include screenshots or diagrams only when they significantly aid understanding
- Ensure images are optimized for both light and dark themes
- Provide text descriptions for visual elements

### Mobile and Device Compatibility

- Format content to be readable on mobile devices
- Use responsive tables or consider alternatives for complex tabular data
- Keep line lengths reasonable for various screen sizes
- Test README rendering on different platforms (GitHub, GitLab, etc.)

## Content Organization

### Recommended Section Order

```markdown
# Project Name

[Badges]

Brief description and key value proposition.

## Table of Contents
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Documentation](#documentation)
- [Contributing](#contributing)
- [License](#license)

## Installation
[Installation instructions]

## Quick Start
[Basic usage example]

## Documentation
[Links to comprehensive documentation]

## Contributing
[How to contribute]

## License
[License information]
```

### Section Guidelines

#### Project Description

- **First paragraph**: Concise elevator pitch (1-2 sentences)
- **Second paragraph**: Key features or capabilities
- **Third paragraph**: Primary use cases or target audience

#### Table of Contents

- Include for READMEs longer than 200 lines
- Use anchor links for navigation within the README
- List major sections in logical order

#### Installation Section

- Start with the simplest installation method
- Include prerequisites and system requirements
- Provide troubleshooting for common installation issues
- Link to detailed installation documentation for complex cases

#### Quick Start Section

- Provide immediate value with a working example
- Show the most common or impressive use case
- Keep examples short but meaningful
- End with links to more comprehensive tutorials

## Templates and Examples

### Library/Package Template

````markdown
# [Package Name]

[![Build Status](badge-url)](workflow-url)
[![License](license-badge)](LICENSE)
[![Python Version](python-badge)](python-url)

Brief description of what this package does and its main value proposition.

## Installation

```bash
pip install [package-name]
```

## Quick Start

```python
from [package] import [main_class]

# Basic usage example
example = [main_class]()
result = example.method("input")
print(result)  # Expected output
```

## Documentation

📚 **[Complete Documentation](docs-url)**

- 🚀 [Getting Started](getting-started-url)
- 📖 [API Reference](api-url)
- 🤝 [Contributing](contributing-url)

## License

This project is licensed under the AGPL-3.0 License - see the [LICENSE](LICENSE) file for details.
````

### CLI Tool Template

````markdown
# [Tool Name]

[![Build Status](badge-url)](workflow-url)
[![License](license-badge)](LICENSE)

Brief description of what this CLI tool does and its main purpose.

## Installation

```bash
pip install [tool-name]
```

## Usage

```bash
# Basic command
[tool-name] [action] [options]

# Example
[tool-name] process --input file.txt --output result.txt
```

## Documentation

📚 **[Complete Documentation](docs-url)**

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

AGPL-3.0 - see [LICENSE](LICENSE) file.
````

## Quality Assurance

### Review Checklist

Before publishing a README, verify:

- [ ] **Core questions answered**: What, Why, How, Who
- [ ] **Installation instructions tested** on clean environment
- [ ] **Code examples verified** and runnable
- [ ] **Links functional** and pointing to correct resources
- [ ] **Badges accurate** and automatically updated
- [ ] **Grammar and spelling** checked
- [ ] **Accessibility considered** in formatting and language
- [ ] **Mobile rendering** tested
- [ ] **Organization standards** followed consistently

### Common Issues to Avoid

**Content Issues**:

- Overly technical language without context
- Missing or outdated installation instructions
- Broken or outdated links
- Code examples that don't work
- Assumptions about user knowledge or environment

**Formatting Issues**:

- Inconsistent heading hierarchy
- Poor mobile device rendering
- Missing syntax highlighting on code blocks
- Overuse of badges or visual elements
- Tables that don't render well on small screens

**Maintenance Issues**:

- Stale badges or status information
- References to deprecated features or documentation
- Outdated version or compatibility information
- Missing updates after major project changes

### Maintenance Guidelines

- **Review quarterly** for accuracy and relevance
- **Update immediately** when installation procedures change
- **Verify links** as part of regular maintenance cycles
- **Test examples** when code interfaces change
- **Update badges** when CI/CD workflows are modified

## Related Guidelines

- See [Documentation Style Guide](./documentation-style-guide.instructions.md) for detailed writing standards
- See [Documentation Structure](./documentation-structure.instructions.md) for comprehensive documentation organization
- See [MkDocs Setup](./documentation-mkdocs-setup.instructions.md) for documentation site integration
