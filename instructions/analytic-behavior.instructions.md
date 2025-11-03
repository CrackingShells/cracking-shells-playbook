---
type: "always_apply"
---

# Codebase Analysis Protocol

## Analysis-First Approach
**Always perform comprehensive analysis before taking action.** The user prioritizes deep understanding over quick solutions.
At the same time **do not overdo it**. The reports must hit the sweet spot of technical consiseness that says it all with clarity.

**Reference**: Follow [Research and Analysis Guidelines](.github/instructions/research-analysis.md) for comprehensive methodology.

## Two-Phase Work Process

### Phase 1: Analysis and Documentation
- **Deep dive first**: Thoroughly study the requested topic before proposing solutions
- **Report-focused iterations**: User may request multiple analysis reports to refine understanding before implementation
- **Expert-level documentation**: All reports must meet senior software engineering standards with sharp insights on the topic
- **Quality over speed**: Prioritize thorough analysis over rapid responses. You can iterate by yourself before writing the reports

### Phase 2: Implementation with Context Refresh
Before writing any code:
1. **Review existing analysis reports** from the current work session
2. **Re-read relevant documentation** and source files
3. **Refresh context** about the specific components you'll modify
4. **Verify current state** of the codebase matches your understanding

## Report Requirements

### Context Precision
Include specific references in all reports:
- **Exact file paths**: `src/components/UserService.ts`
- **Function/class names**: `class UserAuthenticator`, `function validateCredentials()`
- **Line numbers**: When referencing specific code locations
- **Variable names**: Exact identifiers used in the codebase
- **Dependencies**: Related modules, packages, or external services

### Content Standards
- **Comprehensive scope**: Cover all relevant architectural aspects of the topic
- **Targeted Technical depth**: Provide implementation-level insights as code relevant code snippets
- **Cross-references**: Link related components and their interactions
- **Impact analysis**: Explain how changes affect other parts of the system

## Analysis Workflow

### Before Code Changes
1. **Read all relevant source files**
2. **Identify dependencies and relationships**
3. **Document current implementation patterns**
4. **Note potential impact areas**
5. **Create or update analysis reports**

### During Implementation
1. **Reference your analysis reports** for context
2. **Verify assumptions** against current codebase state
3. **Maintain consistency** with documented patterns
4. **Update reports** if discoveries change your understanding

## Memory Management
- **Document decisions**: Record why specific approaches were chosen
- **Maintain context**: Keep detailed notes for complex analysis sessions
- **Cross-reference**: Link related findings across multiple reports
- **Version awareness**: Note when analysis was performed relative to code changes