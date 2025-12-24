# Roadmap Composition Annex: Examples

## 1. Mode 1: The Task Plan (Bug Fix)

# Task 1: Resolve Threading Deadlock
**Goal**: Prevent hang during shutdown of the `ThreadedFileWriter`.
**Pre-conditions**:
- [ ] Deadlock reproduced in `tests/test_shutdown.py`.
**Success Gates**:
- ✅ `tests/test_shutdown.py` passes 50/50 iterations.

## Implementation Steps

### Step 1.1: Create Reproduction Test
- **Goal**: Establish a baseline for the bug.
- **Pre-conditions**: Branch `fix/deadlock` checked out.
- **Consistency Checks**: `pytest tests/test_shutdown.py` (Expected: Fail).
- **Commit**: `test: add reproduction case for writer shutdown deadlock`

### Step 1.2: Implement Timeout Join
- **Goal**: Ensure threads are joined with a timeout to prevent hanging.
- **Pre-conditions**: Step 1.1 complete.
- **Consistency Checks**: `pytest tests/test_shutdown.py` (Expected: Pass).
- **Commit**: `fix(core): add timeout to thread join in writer shutdown`

---

## 2. Mode 2: The Milestone Plan (Feature)

# Milestone 1: OAuth2 Authentication
**Target Version**: v1.1.0-beta

## Task 1.1: Database Migration
**Goal**: Setup tables for OAuth2 providers.
**Pre-conditions**: Database is accessible and migrations are clean.
**Success Gates**:
- ✅ `Provider` and `OAuthToken` tables exist in schema.

### Step 1.1.1: Create Migration Script
- **Goal**: Generate SQL/ORM migration for providers.
- **Pre-conditions**: None.
- **Consistency Checks**: `alembic check`.
- **Commit**: `feat(db): add oauth2 provider and token tables`

## Task 1.2: GitHub Provider Implementation
**Goal**: Implement GitHub-specific OAuth2 flow.
**Success Gates**:
- ✅ User can login via `/login/github`.

### Step 1.2.1: Implement Redirect Route
- **Goal**: Redirect user to GitHub auth page.
- **Pre-conditions**: Task 1.1 complete.
- **Consistency Checks**: `curl -I localhost:8000/login/github` (Expected: 302).
- **Commit**: `feat(auth): implement github oauth2 redirect endpoint`

---

## 3. Mode 3: The Campaign Roadmap (Strategic)

# Phase 1: Infrastructure Modernization

### Milestone 1.1: Containerization
**Target Version**: v0.5.0

#### Task 1.1.1: Dockerfile Optimization
**Goal**: Reduce image size below 500MB.
**Pre-conditions**: Current image size measured > 1GB.
**Success Gates**:
- ✅ `docker images` shows size < 500MB.

##### Step 1.1.1.1: Multi-stage Build
- **Goal**: Separate build and runtime stages.
- **Pre-conditions**: Dockerfile exists.
- **Consistency Checks**: `docker build . -t app:latest`.
- **Commit**: `perf(docker): implement multi-stage build`
