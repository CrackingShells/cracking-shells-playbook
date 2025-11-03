---
applyTo: 'docs/**/*.md'
description: 'Documentation style guide for content and writing standards.'
---

# Documentation Style Guide

## Overview
This file defines the writing style, tone, and content standards for all documentation in the `docs/` directory.

## Style Requirements

### Tone Standards
- Maintain focused professionalism and technical clarity throughout.
- Use warmth or engagement only at crucial junctions (e.g., transitions, summaries, or onboarding moments), not as a default.
- Be humble and educational.

### Compelling Conciseness
- Documentation is the readable front-end description of the project. It must be pleasant to read to keep users hooked and focused. There is a sweet spot between dryness of technical writing and engaging writing.
- Use precise paragraphs that tell the reader exactly what they need to know about topics.
- Apply DRY (Don't Repeat Yourself) principles:
  - Avoid repeating information.
  - Rely on cross-linking all the documentation.
  - Merge similar articles to reduce redundancy.
- Avoid explanations or recommendations that do not stem from the project.

### Language Standards
- Use plain, neutral language.
- Avoid subjective statements or recommendations.
- Prefer present tense and active voice.
- Use consistent terminology throughout all documentation.

### Audience Guidelines
- Main articles are for all users, but assume some technical background.
- Direct beginners to appendices for foundational concepts.
- Clearly distinguish between user-facing and developer-facing content.

## Content Organization

### Article Introduction Pattern
Introduce technical articles with the content it covers:
```markdown
This article is about:
- <short list of concepts discussed>
```

### Content Structure Requirements
- Each article should focus on a single topic or feature.
- Use headings for logical organization.
- Use code snippets to illustrate points or provide examples.
- Reference resources (images, diagrams) to support text.
- Do not repeat information from code docstrings—link or refer to code as needed.

### Cross-Reference Standards
- Link to related articles and appendices consistently.
- Use relative paths for internal documentation links.
- Provide context when linking to external resources.
- Maintain link integrity during content updates.

## Content Categories

### User Documentation (`docs/articles/users/`)
- Getting started guides and tutorials
- Feature-specific documentation and usage examples
- End-user troubleshooting and FAQ content
- Product-specific guides and workflows

### Developer Documentation (`docs/articles/devs/`)
- Architecture documentation and design decisions
- Contribution guidelines and development processes
- Implementation guides and technical specifications
- CI/CD processes and development tools

### API Documentation (`docs/articles/api/`)
- Auto-generated API reference from docstrings
- Usage examples and integration patterns
- Module-level documentation and architecture
- Code examples and best practices

### Appendices (`docs/articles/appendices/`)
- Glossary of terms and definitions
- Foundational concepts and background information
- Troubleshooting guides and common issues
- Legacy documentation and migration guides

## Quality Standards

### Accuracy Requirements
- Keep documentation accurate and aligned with the latest released code.
- Verify all code examples and snippets work correctly.
- Update documentation immediately when APIs or behaviors change.
- Test all links and references regularly.

### Consistency Requirements
- Use consistent formatting and structure across all files.
- Follow established naming conventions and terminology.
- Maintain consistent navigation and cross-referencing patterns.
- Apply the same style standards across all content types.

## Content Maintenance

### Update Guidelines
- Prefer updating existing files in-place rather than creating new files when features change, to preserve inbound links and avoid fragmentation of content.
- When a behavior or API is removed or intentionally deprecated, mark the relevant section clearly with a short note and an optional migration snippet, but keep the original page available for historical context unless it causes confusion.
- If a section becomes entirely obsolete, add a short deprecation banner at the top and link to the replacement content; consider archiving the page in an `archive/` folder rather than deleting it.

### Automated Update Process
- Use the `prompts/update_documentation.prompt.md` workflow to guide automated updates after code changes.
- The prompt should instruct the agent to edit existing pages in-place where possible, preserve or update links, and add a short changelog entry at the top of any edited file.

## Related Guidelines
- See [Documentation Structure](./documentation-structure.instructions.md) for directory organization standards
- See [MkDocs Setup](./documentation-mkdocs-setup.instructions.md) for technical configuration
- See [API Documentation](./documentation-api.instructions.md) for automated documentation generation
- See [Documentation Resources](./documentation-resources.instructions.md) for asset management