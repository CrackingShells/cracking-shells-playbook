---
applyTo: 'mkdocs.yml, .readthedocs.yaml, docs/requirements.txt, pyproject.toml'
description: 'MkDocs setup and configuration standards for automated documentation generation.'
---

# MkDocs Setup Instructions

## Overview
All repositories must implement automated documentation generation using MkDocs with mkdocstrings for consistent, professional documentation that auto-updates with code changes.

## Required Configuration Files

### 1. MkDocs Configuration (`mkdocs.yml`)
Create this file in the repository root with the following standardized structure:

```yaml
site_name: [Repository Name]
site_description: [Brief repository description]
site_url: https://crackingshells.github.io/[Repository-Name]/
repo_url: https://github.com/CrackingShells/[Repository-Name]
repo_name: CrackingShells/[Repository-Name]
docs_dir: docs

theme:
  name: material
  features:
    - content.code.copy

plugins:
  - search
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
  - print-site

markdown_extensions:
  - admonition
  - tables
  - fenced_code
  - toc:
      permalink: true

nav:
  - Home: index.md
  - Users:
      - Getting Started: articles/users/GettingStarted.md
      # Add user-facing documentation sections
  - Developers:
      - Overview: articles/devs/index.md
      # Add developer documentation sections
  - API Reference:
      - Overview: articles/api/index.md
      # Add API documentation sections organized by module/package
  - Appendices:
      - Overview: articles/appendices/index.md
      - Glossary: articles/appendices/glossary.md
```

#### Configuration Details

**Site Information**:
- Replace `[Repository Name]` with the actual repository name
- Replace `[Repository-Name]` with the GitHub repository name
- Use descriptive site descriptions that explain the project's purpose

**Theme Configuration**:
- Uses Material theme for professional appearance
- `content.code.copy` enables copy buttons on code blocks
- Additional features can be added as needed

**Plugin Configuration**:
- `search`: Enables full-text search across documentation
- `mkdocstrings`: Automated API documentation generation from docstrings
- `print-site`: Enables PDF generation of entire documentation

**Markdown Extensions**:
- `admonition`: Support for callout boxes and warnings
- `tables`: Enhanced table support
- `fenced_code`: Code blocks with syntax highlighting
- `toc`: Table of contents with permalinks

### 2. ReadTheDocs Configuration (`.readthedocs.yaml`)
Create this file in the repository root:

```yaml
# Read the Docs configuration file
# See https://docs.readthedocs.io/en/stable/config-file/v2.html for details

# Required
version: 2

# Set the OS, Python version, and other tools you might need
build:
  os: ubuntu-24.04
  tools:
    python: "3.13"

# Build documentation with Mkdocs
mkdocs:
   configuration: mkdocs.yml

# Optionally, but recommended,
# declare the Python requirements required to build your documentation
# See https://docs.readthedocs.io/en/stable/guides/reproducible-builds.html
python:
   install:
   - requirements: docs/requirements.txt
```

#### ReadTheDocs Details

**Build Environment**:
- Ubuntu 24.04 for consistent, modern environment
- Python 3.13 for latest language features
- Standardized across all organization repositories

**Build Configuration**:
- References the `mkdocs.yml` configuration file
- Installs Python dependencies from `docs/requirements.txt`
- Enables reproducible builds with pinned dependencies

### 3. Documentation Dependencies (`docs/requirements.txt`)
Create this file with the required MkDocs packages:

```
mkdocstrings
mkdocstrings-python
mkdocs-material
mkdocs-print-site-plugin
```

#### Package Descriptions

- **mkdocstrings**: Core plugin for automatic API documentation generation
- **mkdocstrings-python**: Python-specific handlers for docstring processing
- **mkdocs-material**: Modern, responsive Material Design theme
- **mkdocs-print-site-plugin**: PDF generation capability for entire site

#### Version Considerations
- Use unpinned versions to get latest compatible releases
- Pin specific versions only if compatibility issues arise
- Update dependencies regularly for security and feature improvements

### 4. Development Dependencies (`pyproject.toml`)
Add optional documentation dependencies to your project configuration:

```toml
[project.optional-dependencies]
docs = [ "mkdocs>=1.4.0", "mkdocstrings[python]>=0.20.0" ]
```

#### Purpose
- Enables local development and testing of documentation
- Allows developers to build documentation without separate dependency management
- Integrates with standard Python development workflows

## Local Development Setup

### Installing Dependencies
Repositories should support local documentation development with either approach:

**Option 1: Using project dependencies**
```bash
pip install -e .[docs]
```

**Option 2: Using documentation requirements directly**
```bash
pip install -r docs/requirements.txt
```

### Development Commands

**Serve documentation with live reloading:**
```bash
mkdocs serve
```
- Starts local server on `http://127.0.0.1:8000`
- Automatically reloads when files change
- Ideal for iterative development and testing

**Build static documentation:**
```bash
mkdocs build
```
- Generates static HTML in `site/` directory
- Useful for testing production builds locally
- Required for deployment validation

**Clean build artifacts:**
```bash
mkdocs build --clean
```
- Removes existing build artifacts before building
- Ensures clean builds without stale content

### Development Workflow

1. **Make documentation changes** in appropriate files
2. **Run `mkdocs serve`** to test changes locally
3. **Verify functionality** including:
   - Navigation works correctly
   - Links are functional
   - API documentation generates properly
   - Images and resources load correctly
4. **Test production build** with `mkdocs build`
5. **Commit changes** once local testing passes

## ReadTheDocs Integration

### Automatic Publishing
- Documentation automatically builds and publishes on repository changes
- All repositories should have ReadTheDocs projects configured
- Documentation URLs follow pattern: `https://crackingshells.github.io/[Repository-Name]/`

### Build Environment Standards
- **Python Version**: 3.13 for consistency across all projects
- **Operating System**: Ubuntu 24.04 for modern, stable environment
- **Dependency Management**: Through `docs/requirements.txt` for reproducible builds
- **Build Tools**: MkDocs with standardized plugin configuration

### Error Handling
- ReadTheDocs provides detailed build logs for debugging
- Common issues include missing dependencies, broken links, or malformed configuration
- Build failures prevent documentation updates, ensuring quality control

### Integration Steps

1. **Configure ReadTheDocs Project**:
   - Link GitHub repository to ReadTheDocs
   - Enable automatic builds on commits
   - Configure custom domain if needed

2. **Test Integration**:
   - Make a test commit to trigger build
   - Verify documentation builds successfully
   - Check that URLs and navigation work correctly

3. **Monitor Build Status**:
   - Set up notifications for build failures
   - Regularly check build logs for warnings
   - Update dependencies when security updates are available

## Quality Assurance

### Configuration Validation
- Test `mkdocs.yml` syntax and structure
- Verify all navigation paths point to existing files
- Ensure plugin configurations are correct and functional

### Build Testing
- Test both development and production builds locally
- Verify that all features (search, API docs, PDF generation) work
- Check responsive design and mobile compatibility

### Content Validation
- Ensure all links work correctly
- Verify images and resources load properly
- Test API documentation generation with sample modules
- Validate navigation structure and user experience

## Troubleshooting

### Common Issues

**Build Failures**:
- Check `docs/requirements.txt` for missing or incompatible packages
- Verify Python version compatibility
- Review ReadTheDocs build logs for specific error messages

**Missing API Documentation**:
- Ensure Python modules are properly imported
- Check docstring format (must be Google-style)
- Verify mkdocstrings configuration in `mkdocs.yml`

**Navigation Problems**:
- Check file paths in `mkdocs.yml` navigation section
- Ensure all referenced files exist
- Verify relative path accuracy

**Theme Issues**:
- Update mkdocs-material to latest version
- Check theme configuration for deprecated options
- Verify browser compatibility

### Debug Strategies
1. **Test locally first** with `mkdocs serve`
2. **Check build logs** for specific error messages
3. **Isolate issues** by testing individual components
4. **Compare with working configurations** from other repositories

## Migration Guidelines

### From Existing Documentation Systems
1. **Preserve content structure** as much as possible
2. **Create redirect mappings** for changed URLs
3. **Test all existing links** and update as necessary
4. **Migrate assets** to `docs/resources/` directory
5. **Update cross-references** to match new structure

### Incremental Migration
- Implement MkDocs configuration without removing existing documentation
- Test new system alongside existing system
- Gradually migrate content sections
- Update links and references incrementally
- Switch over only when new system is fully validated

## Related Guidelines
- See [Documentation Structure](./documentation-structure.instructions.md) for directory organization
- See [API Documentation](./documentation-api.instructions.md) for automated API docs
- See [Documentation Style Guide](./documentation-style-guide.instructions.md) for writing standards
- See [Documentation Resources](./documentation-resources.instructions.md) for asset management