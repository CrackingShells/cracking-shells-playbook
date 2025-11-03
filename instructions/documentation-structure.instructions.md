---
applyTo: 'docs/**/*'
description: 'Documentation directory structure and organization standards.'
---

# Documentation Structure Standards

## Overview
This file defines the standardized directory structure, file organization, and navigation patterns for all documentation.

## Standard Directory Structure

```
docs/
├── index.md                          # Main landing page
├── requirements.txt                  # Build dependencies (for MkDocs)
├── articles/
│   ├── index.md                     # Main landing page for articles with links to key sections
│   ├── users/                       # User-facing documentation
│   │   ├── GettingStarted.md       # Required: Getting started guide
│   │   ├── tutorials/              # Step-by-step tutorials
│   │   │   ├── Topic1/
│   │   │   │   ├── <tutorial_step1>.md
│   │   │   │   └── <tutorial_step2>.md
│   │   │   └── Topic2/
│   │   │       └── <tutorial_a_unique_file_if_short>.md
│   │   ├── <article_name>.md       # Feature-specific articles
│   │   ├── Feature1/               # Feature-specific directories
│   │   │   ├── <article_name1>.md
│   │   │   └── <article_name2>.md
│   │   └── Feature2/
│   │       └── <article_name>.md
│   ├── devs/                        # Developer-facing documentation
│   │   ├── index.md                # Developer overview
│   │   ├── architecture/           # System architecture
│   │   │   ├── <article_name>.md
│   │   │   └── <other_architecture_articles>.md
│   │   ├── contribution_guidelines/ # How to contribute
│   │   │   ├── <article_name>.md
│   │   │   └── <other_contribution_articles>.md
│   │   ├── ci_cd/                  # CI/CD processes
│   │   │   ├── <article_name>.md
│   │   │   └── <other_ci_cd_articles>.md
│   │   └── implementation_guides/  # Technical implementation guides
│   ├── api/                        # API Reference (auto-generated)
│   │   ├── index.md                # API overview and getting started
│   │   ├── <module_name>.md        # Individual module documentation
│   │   └── <subpackage>/           # Subpackage organization
│   │       ├── <submodule>.md      # Submodule documentation
│   │       └── index.md            # Subpackage overview
│   └── appendices/                 # Supporting documentation
│       ├── index.md                # Appendices overview with TOC
│       ├── glossary.md             # Required: Glossary of terms
│       └── <other_appendix_articles>.md
└── resources/                      # Non-markdown assets
    ├── diagrams/                   # PlantUML diagrams
    │   └── <diagram_name>.puml
    └── images/                     # Images and other assets
        ├── <image_name>.png
        └── <other_resources>.jpg
```

## File Naming Conventions

### General Rules
- Use PascalCase for main article files (e.g., `GettingStarted.md`)
- Use lowercase with hyphens for directories (e.g., `contribution-guidelines/`)
- Use descriptive, meaningful names that clearly indicate content
- Avoid special characters, spaces, and non-ASCII characters

### Specific Patterns
- **Tutorial files**: Use numbered prefixes for sequential content (e.g., `01-installation.md`, `02-configuration.md`)
- **Feature articles**: Use descriptive names matching the feature (e.g., `PackageManagement.md`, `EnvironmentConfiguration.md`)
- **API documentation**: Match the module name (e.g., `cli_hatch.py` → `cli.md`)

## Content Organization Patterns

### User Documentation (`docs/articles/users/`)
Content for API/product consumers:
- **Getting Started**: Required entry point for new users
- **Tutorials**: Step-by-step walkthroughs organized by topic
  - See [Tutorial Guidelines](./tutorials.instructions.md) for detailed conventions
  - Keep organized by topic and section
  - Link back to glossary and appendices when needed
- **Feature Documentation**: Focused articles for specific features
- **Use subcategories for related articles** (e.g., `RelevantCategoryForTheProduct/`)
- **Use subcategories for different products** (e.g., `AnotherCategoryForTheProduct/`)

### Developer Documentation (`docs/articles/devs/`)
Content for maintainers and code contributors:

#### Contribution Guidelines
- How to contribute (issues, pull requests, etc.)
- How to run tests, build the project, and deploy
- How to enhance the project (e.g., adding features, improving performance)

#### Code Structure and Architecture
- What directories and/or files are present in the codebase
- Design patterns used, where, for what gains
- Extensive use of diagrams is encouraged

#### CI and CD Processes
- How the CI/CD is set up
- What tools are used (e.g., GitHub Actions)
- Description of the CI/CD pipeline
- Extensive use of diagrams is encouraged

### API Documentation (`docs/articles/api/`)
Auto-generated documentation from code:
- **Overview page**: Context and getting started information
- **Module pages**: One per main module using mkdocstrings
- **Subpackage organization**: Logical grouping of related modules
- See [API Documentation Guidelines](./documentation-api.instructions.md) for details

### Appendices (`docs/articles/appendices/`)
Supporting and supplementary information:

#### Required Content
- **Table of contents**: Required for appendices that links every article in the appendices folder
- **Glossary**: Terms and definitions in alphabetical order, separated by first letter sections

#### Supplementary Content
- Beginner resources and foundational concepts
- Step-by-step guides for basic tasks (e.g., opening a terminal on Windows/Mac)
- Any content that would otherwise clutter the main articles
- Legacy documentation and migration guides

## Navigation Structure Requirements

### MkDocs Navigation Pattern
All repositories must implement this navigation structure in `mkdocs.yml`:

```yaml
nav:
  - Home: index.md
  - Users:
      - Getting Started: articles/users/GettingStarted.md
      - [Feature Areas]: articles/users/[FeatureArea]/
  - Developers:
      - Overview: articles/devs/index.md
      - Architecture: articles/devs/architecture/
      - Contribution Guides: articles/devs/contribution_guidelines/
      - Implementation Guides: articles/devs/implementation_guides/
  - API Reference:
      - Overview: articles/api/index.md
      - [Core Modules]: articles/api/[module].md
      - [Subpackages]: articles/api/[subpackage]/
  - Appendices:
      - Overview: articles/appendices/index.md
      - Glossary: articles/appendices/glossary.md
```

### Standard Navigation Sections
1. **Home**: Landing page (`index.md`)
2. **Users**: User-facing documentation
   - Getting Started guide (required)
   - User tutorials and guides
   - Feature-specific documentation
3. **Developers**: Developer-facing documentation
   - Architecture documentation
   - Contribution guidelines
   - Implementation guides
4. **API Reference**: Automated API documentation
   - Organized by logical module groupings
   - Index page for context
5. **Appendices**: Supporting information
   - Glossary (required)
   - Troubleshooting guides
   - Known limitations

## Cross-Reference Standards

### Internal Linking
- Use relative paths for all internal documentation links
- Maintain consistent link patterns across all files
- Provide meaningful link text that describes the destination
- Link to related content to create a cohesive documentation web

### External References
- Clearly indicate external links with appropriate context
- Use stable URLs when possible
- Consider link longevity and maintenance requirements

## File Organization Best Practices

### Content Grouping
- Group related content in subdirectories
- Keep similar content types together
- Use logical hierarchies that match user mental models
- Balance depth vs. breadth in directory structures

### Index Pages
- Provide index pages for major sections
- Include overview content and navigation aids
- Link to all sub-content from index pages
- Use index pages to provide context and orientation

## Migration and Evolution

### Backward Compatibility
- Preserve existing URLs when restructuring content
- Use redirects or clear migration notices when URLs change
- Maintain historical context for deprecated content
- Plan migration paths for major structural changes

### Growth Planning
- Design directory structures to accommodate future growth
- Leave room for expansion in navigation hierarchies
- Consider how new content types will fit into existing patterns
- Plan for internationalization and multi-language support if needed

## Related Guidelines
- See [Documentation Style Guide](./documentation-style-guide.instructions.md) for writing standards
- See [MkDocs Setup](./documentation-mkdocs-setup.instructions.md) for technical configuration
- See [API Documentation](./documentation-api.instructions.md) for automated documentation
- See [Documentation Resources](./documentation-resources.instructions.md) for asset management