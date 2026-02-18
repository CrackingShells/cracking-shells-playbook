# Git Workflow: Milestone-Based Development

For projects with formal roadmaps, use a flat branching model where all task branches come from and merge back to a single milestone branch. The roadmap directory tree governs execution order — git stays flat.

## Branch Hierarchy

```plaintext
main (production releases only)
  └── dev (development integration branch)
      └── milestone/<campaign>
          ├── task/<descriptive-name>
          ├── task/<descriptive-name>
          ├── task/<descriptive-name>
          └── ...
```

## Workflow Rules

1. **Campaign branch from `dev`**: `milestone/<campaign>` is the single integration branch for the entire campaign.

2. **Task branches from milestone**: Every leaf task file in the roadmap gets a `task/<descriptive-name>` branch created from `milestone/<campaign>`. Branch names use kebab-case versions of the roadmap filesystem names.

3. **Breadth-first merge order**: Agents create and work on task branches following the roadmap tree's breadth-first order. All depth-d sibling leaves can be branched and worked in parallel. They must all merge back to milestone before depth d+1 task branches are created.

4. **Merge back to milestone**: When a task's success gates are met, its branch merges directly into `milestone/<campaign>`.

5. **Merge criteria**:
   - **Task → Milestone**: Task success gates met, step consistency checks pass
   - **Milestone → dev**: All campaign nodes done, full test suite passes, final agent review
   - **dev → main**: Release-ready, all tests pass

6. **Conventional Commits**: Follow organization's conventional commit format to enable automated semantic versioning

## Example Workflow

```bash
# Start campaign
git checkout dev
git checkout -b milestone/cli-ux-normalization

# Work on depth-0 leaves (parallel)
git checkout -b task/test-setup
git commit -m "test(cli): add test infrastructure setup"
git checkout milestone/cli-ux-normalization
git merge task/test-setup
git branch -d task/test-setup

# All depth-0 leaves merged, proceed to depth-1
git checkout -b task/color-system
git commit -m "test(cli): add color system tests"
git commit -m "feat(cli): implement color system"
git checkout milestone/cli-ux-normalization
git merge task/color-system
git branch -d task/color-system

# When all campaign work done, merge to dev
git checkout dev
git merge milestone/cli-ux-normalization
git branch -d milestone/cli-ux-normalization
```

## Benefits
- Flat branch model — no nested component branches to manage
- Execution order governed by roadmap tree, not branch hierarchy
- Parallel task branches for sibling leaves at each depth
- Clear merge criteria at each level
- Git history reflects breadth-first execution order
- Enables automated changelog generation per campaign

**Cross-References**:
- See `roadmap-generation.instructions.md` for the directory-tree roadmap model
- See `code-change-phases.instructions.md` for the 3-stage workflow
- See `testing.instructions.md` for test requirements at each merge level
