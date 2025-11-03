---
applyTo: 'docs/**/*.md'
description: 'Master documentation guidelines with references to specialized instruction files.'
---

# Documentation Instructions Overview

## Philosophy
Documentation is the readable front-end description of the project. It must be pleasant to read, focused, and technically accurate. All documentation must follow organization standards for consistency and maintainability.

## Documentation Standards System

This documentation system is organized into specialized instruction files that cover different aspects of documentation creation and maintenance:

### 📝 [Style Guide](./documentation-style-guide.instructions.md)
**Applies to**: `docs/**/*.md`
- Writing tone, language, and audience guidelines
- Content organization and cross-referencing standards
- Quality assurance and maintenance procedures

### 🏗️ [Structure Standards](./documentation-structure.instructions.md) 
**Applies to**: `docs/**/*`
- Directory organization and file naming conventions
- Navigation patterns and content categorization
- User/developer/API content separation

### ⚙️ [MkDocs Setup](./documentation-mkdocs-setup.instructions.md)
**Applies to**: `mkdocs.yml`, `.readthedocs.yaml`, `docs/requirements.txt`, `pyproject.toml`
- Complete MkDocs configuration templates
- ReadTheDocs integration and build environment
- Local development workflow and troubleshooting

### 🔧 [API Documentation](./documentation-api.instructions.md)
**Applies to**: `docs/articles/api/**/*.md`, `**/*.py`
- Automated API documentation generation using mkdocstrings
- Google-style docstring requirements and examples
- Module organization and navigation integration

### 🎨 [Resources Management](./documentation-resources.instructions.md)
**Applies to**: `docs/resources/**/*`
- PlantUML diagram standards and examples
- Image management and organization guidelines
- Asset optimization and accessibility standards

## Quick Start Guide

### For New Repositories
1. **Set up MkDocs**: Follow [MkDocs Setup Instructions](./documentation-mkdocs-setup.instructions.md)
2. **Create directory structure**: Use [Structure Standards](./documentation-structure.instructions.md)
3. **Write initial content**: Apply [Style Guide](./documentation-style-guide.instructions.md)
4. **Add API documentation**: Follow [API Documentation](./documentation-api.instructions.md)
5. **Include resources**: Use [Resources Management](./documentation-resources.instructions.md)

### For Existing Repositories
1. **Review current documentation** against specialized guidelines
2. **Plan migration** following structure and setup standards
3. **Implement MkDocs** configuration using provided templates
4. **Migrate content** preserving URLs and cross-references
5. **Add automated API documentation** for improved maintenance

### For Content Updates
1. **Identify the relevant specialized guideline** for your changes
2. **Follow the specific standards** for that content type
3. **Test locally** using MkDocs development workflow
4. **Maintain cross-references** and navigation consistency

## Core Principles

### Universal Standards
- **Markdown format** for all documentation files
- **Focused content** that tells readers exactly what they need to know
- **DRY principles** with cross-linking instead of repetition
- **Technical accuracy** aligned with latest code
- **Professional tone** with warmth only at crucial junctions

### Organization Requirements
- **Consistent structure** across all repositories
- **Clear separation** between user, developer, and API content
- **Logical navigation** that matches user mental models
- **Automated generation** where possible to reduce maintenance

### Quality Assurance
- **Local testing** required before committing changes
- **Cross-reference validation** to ensure link integrity
- **Accessibility standards** for all visual content
- **Regular maintenance** to keep content current

## Implementation Status

### Established Repositories
- **Hatch**: ✅ Full MkDocs implementation with API documentation
- **Hatchling**: 🔄 Migration needed from Jekyll to MkDocs standards

### Template Resources
All specialized instruction files include:
- Complete configuration templates
- Working examples from established implementations
- Step-by-step setup procedures
- Troubleshooting guides and best practices

## Support and Resources

### Getting Help
- **Configuration issues**: See [MkDocs Setup](./documentation-mkdocs-setup.instructions.md) troubleshooting
- **Content organization**: Reference [Structure Standards](./documentation-structure.instructions.md)
- **Writing questions**: Follow [Style Guide](./documentation-style-guide.instructions.md)
- **Technical problems**: Check individual instruction files for specific guidance

### Contributing to Documentation Standards
- Propose changes through standard pull request process
- Test changes against existing implementations
- Update all affected specialized instruction files
- Maintain backward compatibility during transitions

---

**System Status**: ✅ Comprehensive specialized guidelines available  
**Last Updated**: September 24, 2025  
**Coverage**: Style, Structure, MkDocs, API, Resources
