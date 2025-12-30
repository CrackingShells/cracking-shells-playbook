# Git Workflow: Milestone-Based Development

For projects with formal roadmaps and milestone tracking, use hierarchical branching to organize work by phases, milestones, and tasks.

## Branch Hierarchy

```plaintext
main (production releases only)
  └── dev (development integration branch)
      ├── milestone/1.1-short-description
      │   ├── task/1.1.1-short-description
      │   ├── task/1.1.2-short-description
      │   └── task/1.1.3-short-description
      ├── milestone/1.2-short-description
      │   └── task/1.2.1-short-description
      └── milestone/2.1-short-description
          └── task/2.1.1-short-description
```

## Workflow Rules

1. **All work from `dev` branch** (not `main`)
   - `main` is production-only, receives merges only at major releases
   - `dev` is the integration branch for all development work

2. **Milestone branches from `dev`**
   - Branch naming: `milestone/<milestone-id>-<short-description>`
   - Example: `milestone/2.1-thread-safe-architecture`
   - Created when milestone work begins
   - Deleted after merge back to `dev`

3. **Task branches from milestone branches**
   - Branch naming: `task/<task-id>-<short-description>`
   - Example: `task/2.1.1-design-architecture`
   - Created when task work begins
   - Deleted after merge back to milestone branch

4. **Merge Hierarchy**:
   - Task branches → Milestone branch (when task complete)
   - Milestone branch → `dev` (when ALL milestone tasks complete)
   - `dev` → `main` (when ready for production release)

5. **Merge Criteria**:
   - **Task → Milestone**: Task success gates met, task tests pass
   - **Milestone → dev**: All milestone tasks complete, all milestone tests pass, no regressions
   - **dev → main**: ALL tests pass (regression, unit, integration, performance), ready for release

6. **Conventional Commits**: Follow organization's conventional commit format to enable automated semantic versioning

## Example Workflow

```bash
# Start milestone work
git checkout dev
git checkout -b milestone/1.1-core-functionality

# Start task work
git checkout -b task/1.1.1-implement-parser

# Work on task with conventional commits
git commit -m "feat(parser): add basic parsing logic"
git commit -m "test(parser): add parser validation tests"

# Complete task - merge to milestone
git checkout milestone/1.1-core-functionality
git merge task/1.1.1-implement-parser
git branch -d task/1.1.1-implement-parser

# Complete all milestone tasks, merge to dev
git checkout dev
git merge milestone/1.1-core-functionality
git branch -d milestone/1.1-core-functionality

# When ready for release, merge to main
git checkout main
git merge dev
git tag v1.1.0
```

## Benefits
- Clear organization of complex multi-phase projects
- Easy tracking of milestone progress
- Facilitates parallel development on different milestones
- Git history reflects roadmap structure
- Enables automated changelog generation per milestone

**Cross-References**:
- See `roadmap-generation.instructions.md` for milestone/task structure
- See `testing.instructions.md` for test requirements at each merge level
