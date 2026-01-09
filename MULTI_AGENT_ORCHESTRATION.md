# Multi-Agent Orchestration Guide

## Overview

This project uses a multi-agent workflow with two specialized agents:

| Agent | Worktree | Branch | Focus |
|-------|----------|--------|-------|
| **Implementation** | `/workspace/worktree-impl` | `agent/implementation` | Code, fixes, features |
| **Testing** | `/workspace/worktree-test` | `agent/testing` | Tests, verification |

## Shared Resources

### Global Todolist
- Location: `rust/AGENT_TODOLIST.json`
- Both agents read and update this file
- Tasks have `assigned_to` field to indicate ownership

### Progress File
- Location: `rust/progress.txt`
- Persistent learnings and GOTCHAs
- Handoff notes between agents

## Workflow

```
┌──────────────────┐
│  Master Agent    │
│  (Orchestrator)  │
└────────┬─────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌────────┐  ┌────────┐
│  IMPL  │  │  TEST  │
│ Agent  │◄─►│ Agent  │
└───┬────┘  └────┬───┘
    │            │
    ▼            ▼
 worktree-impl  worktree-test
```

## Starting an Agent

### Implementation Agent
```bash
cd /workspace/worktree-impl
cat AGENT_INSTRUCTIONS.md
# Read rust/AGENT_TODOLIST.json for tasks
# Work on implementation
# Update progress.txt
# Commit changes
```

### Testing Agent
```bash
cd /workspace/worktree-test
cat AGENT_INSTRUCTIONS.md
# Read rust/AGENT_TODOLIST.json for tasks
# Write and run tests
# Update progress.txt
# Commit changes
```

## Syncing Changes

### After Implementation Work
```bash
cd /workspace/worktree-impl
git add -A
git commit -m "feat: description"
# Update AGENT_TODOLIST.json
# Add handoff note to progress.txt
```

### After Testing Work
```bash
cd /workspace/worktree-test
git add -A
git commit -m "test: description"
# Update AGENT_TODOLIST.json
```

### Merging Back to Main
```bash
# From main workspace
cd /workspace
git fetch .
git merge agent/implementation --no-commit
git merge agent/testing --no-commit
git commit -m "merge: agent work"
```

## Task States

| Status | Meaning |
|--------|---------|
| `pending` | Not started |
| `in_progress` | Agent is working on it |
| `completed` | Done, has result |
| `blocked` | Waiting on something |

## Handoff Protocol

1. **Impl → Test**: After implementing feature
   - Update progress.txt: `[TIMESTAMP] [IMPL] -> [TEST]: What to test`
   - Mark task as `completed` in todolist

2. **Test → Impl**: When finding bugs
   - Update progress.txt: `[TIMESTAMP] [TEST] -> [IMPL]: Bug description`
   - Create new task with `BUG-` prefix

## Current State

### Worktrees
```
/workspace                - Main workspace (orchestration)
/workspace/worktree-impl  - Implementation agent
/workspace/worktree-test  - Testing agent
```

### Branches
- `cursor/rust-agent-to-do-list-e999` - Main branch
- `agent/implementation` - Implementation agent branch
- `agent/testing` - Testing agent branch

## Commands Reference

```bash
# List worktrees
git worktree list

# Switch to implementation context
cd /workspace/worktree-impl

# Switch to testing context
cd /workspace/worktree-test

# Build in either worktree
cargo build

# Test in either worktree
cargo test

# View todolist
cat rust/AGENT_TODOLIST.json | jq

# View progress
cat rust/progress.txt
```

## Initial Tasks

### Implementation Agent Priority
1. AUDIT-001: Audit TODO_CHECKLIST.md
2. IMPL-001: Fix compiler warnings
3. IMPL-002: Verify CSV output
4. IMPL-003: Verify JSON output

### Testing Agent Priority
1. TEST-002: Unit tests for types
2. TEST-003: Unit tests for constraints
3. TEST-004: Integration tests
4. TEST-005: Property-based tests
