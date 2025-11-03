---
applyTo: '**/*'
description: 'Validated Systematic Workflow Pattern for High-Quality Development'
---

# DEVELOPMENT WORKFLOW DOCUMENTATION

Based on direct implementation experience, this workflow pattern consistently delivered high-quality results:

## Phase 1: Architectural Analysis

1. **Current Codebase State Assessment**
   - Comprehensive code review and dependency analysis
   - Identification of existing patterns and architectural decisions
   - Assessment of technical debt and improvement opportunities

2. **Industry Standards Research**
   - Investigation of established patterns for similar problems
   - Comparison with industry best practices and standards
   - Evaluation of alternative approaches and trade-offs

3. **Critical Assessment of Requirements**
   - Deep analysis of functional and non-functional requirements
   - Identification of edge cases and integration challenges
   - Validation of requirements against technical constraints

4. **Recommended Improvements**
   - Proposal of architectural enhancements and optimizations
   - Documentation of design decisions and rationale
   - Creation of implementation roadmap with clear milestones
   - **For complex projects**: Consider creating a formal roadmap following [roadmap generation guidelines](./roadmap-generation.instructions.md)

5. **Iterative Refinement**
   - Stakeholder feedback integration and requirement clarification
   - Architecture adjustment based on implementation discoveries
   - Continuous validation against original objectives

6. **Analysis Accuracy Verification**
   - **File References**: Confirm all referenced file paths exist and are spelled correctly
   - **Line Numbers**: Validate line number references against current file state (not historical commits)
   - **Code Snippets**: Verify code snippets match actual implementation (copy-paste from files, don't paraphrase)
   - **Configuration Values**: Read configuration files directly rather than inferring from commit messages
   - **Dependencies**: Check actual dependency versions in package files (pyproject.toml, requirements.txt, etc.)
   - **Context Timestamp**: Document when analysis was performed (commit SHA or date) for future reference

### Deliverables

Versioned reports on relevant topics following [org's reporting guidelines](./reporting.instructions.md)

## Phase 2: Comprehensive Test Suite Development

1. **Test-Driven Development**
   - Comprehensive tests definition based on vetted architecture design reports (ADRs)
   - Edge case identification and test coverage validation
   - Mock and integration test development

1. **Unit Test Coverage**
   - Individual component testing with 100% coverage target
   - Mock object creation and dependency isolation
   - Performance and resource utilization testing

2. **Integration Testing**
   - End-to-end workflow validation
   - Cross-component interaction testing
   - Real-world scenario simulation

3. **Edge Case Validation**
   - Boundary condition testing and error scenario validation
   - Platform compatibility and environment variation testing
   - Load testing and performance validation

4. **Iterative Refinement**
   - Stakeholder feedback integration and requirement clarification
   - Tests adjustment based on target use cases 
   - Continuous validation against major design decisions

### Deliverables

A series of versioned test definition reports for every iteration following [org's reporting guidelines](./reporting.instructions.md)

## Phase 3: Core Feature Implementation

1. **Documentation & Test driven implementation**
   - Focus on final design decisions
   - Leverage test suite to bound actual implementation
   - Satisfy guidelines over over-engineering

2. **Incremental Implementation**
   - Small, focused changes with immediate validation
   - Continuous integration and testing at each step
   - Regular checkpoint commits with clear progress markers

3. **Proper Error Handling**
   - Comprehensive exception handling and graceful degradation
   - Input validation and boundary condition management
   - Logging and debugging infrastructure implementation


## Phase 4: Persistent Debugging to 100% Test Pass Rate

1. **Systematic Issue Investigation**
   - Root cause analysis for every test failure
   - Fairly re-assess the couple (test, implementation): which is at fault?
   - Design decision-based reporting
   - Evidence-based debugging with comprehensive logging
   - Iterative refinement until all tests pass

2. **Performance Optimization**
   - Bottleneck identification and resolution
   - Resource utilization optimization
   - Scalability validation and improvement

3. **Quality Assurance**
   - Code review and architectural validation
   - Documentation accuracy verification
   - Deployment readiness assessment

### Deliverables

A series of versioned test execution and validation reports for every iteration following [org's reporting guidelines](./reporting.instructions.md)

## Phase 5: Focused, Logical Git Commits

Refer to [Git Workflow Instructions](./git-workflow.md) for detailed commit message standards.

1. **Commit Strategy Planning**
   - Logical change grouping and dependency analysis
   - Commit message standardization and clarity
   - Rollback strategy and recovery planning

2. **Incremental Commits**
   - Single-purpose commits with clear rationale
   - Conventional commit format adherence
   - Comprehensive commit message documentation

3. **Repository Hygiene**
   - Branch management and merge strategy
   - Conflict resolution and history preservation
   - Tag creation and release preparation

## Phase 6: Documentation Creation and Updates

1. **User Documentation**
   - Feature documentation and usage examples
   - Installation and configuration guides
   - Troubleshooting and FAQ development

2. **Developer Documentation**
   - API documentation and code examples
   - Architecture documentation and design decisions
   - Contribution guidelines and development setup

3. **Knowledge Transfer**
   - Implementation insights and lessons learned
   - Future development enablers and extension points
   - Troubleshooting guides and common issues

### Deliverables

Versioned modification reports following [org's reporting guidelines](./reporting.instructions.md)

## Phase 7: Documentation Commits Separate from Code Changes

1. **Documentation Review**
   - Accuracy validation against current implementation
   - Completeness assessment and gap identification
   - User experience and clarity optimization

2. **Separate Documentation Commits**
   - Documentation changes isolated from code changes
   - Clear commit messages indicating documentation updates
   - Version synchronization and consistency validation