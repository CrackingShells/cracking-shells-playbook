# Git Workflow: Debugging Workflow

When debugging issues, use a dedicated debugging branch to encourage frequent commits without polluting the main development history.

## Workflow

1. **Create debugging branch from current location**:
   ```bash
   git checkout -b debugging/<issue-description>
   # Example: debugging/memory-leak-in-profiler
   ```

2. **Commit frequently during debugging**:
   - Commit every hypothesis test
   - Commit every diagnostic change
   - Commit every attempted fix
   - Use descriptive messages: `debug: test hypothesis X`, `debug: add logging for Y`

3. **When bug is fixed**:

   **Option A: Clean commit on original branch** (recommended for simple fixes)
   ```bash
   # Identify the fix
   git log  # Review debugging commits to understand the fix

   # Switch back to original branch
   git checkout <original-branch>

   # Make ONE clean commit with the fix
   git add <fixed-files>
   git commit -m "fix: resolve memory leak in profiler shutdown

   Root cause: ThreadedFileWriter not properly joining threads
   Solution: Add explicit thread.join() with timeout in shutdown()

   Fixes #123"
   ```

   **Option B: Sequence of clean commits** (for complex fixes)
   ```bash
   # Switch back to original branch
   git checkout <original-branch>

   # Make a sequence of logical commits
   git commit -m "refactor: extract thread cleanup logic"
   git commit -m "fix: add thread join with timeout"
   git commit -m "test: add thread cleanup validation"
   ```

4. **Delete debugging branch**:
   ```bash
   git branch -D debugging/<issue-description>
   ```

## Benefits
- Encourages thorough debugging with frequent commits
- Preserves debugging history for learning (before deletion)
- Keeps main development history clean and logical
- Facilitates root cause analysis documentation

## Debugging Commit Format
```plaintext
debug: <what you're testing/trying>

# Examples:
debug: test hypothesis that memory leak is in file writer
debug: add logging to track thread lifecycle
debug: try alternative shutdown sequence
```

## Final Fix Commit Format
```plaintext
fix: <what was fixed>

Root cause: <why the bug occurred>
Solution: <how it was fixed>

Fixes #<issue-number>
```

**Cross-Reference**: See `work-ethics.instructions.md` for root cause analysis guidelines.
