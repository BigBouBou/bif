# bif

bif is a lazy CLI note-taking app that stores each note as timestamped entry in a .bif file. The app tracks a single .bif file and allows the user to quickly capture thoughts as short entries without leaving the terminal, or taking time to organise the files attributed.

---

## Table of Contents

- [bif](#bif)
  - [Table of Contents](#table-of-contents)
  - [Project Goals](#project-goals)
  - [User Model (What bif is)](#user-model-what-bif-is)
  - [Architecture Overview](#architecture-overview)
  - [Module Responsibilities](#module-responsibilities)
  - [On-Disk Storage Layout](#on-disk-storage-layout)
  - [State Model and Invariants](#state-model-and-invariants)
  - [Command Semantics](#command-semantics)
  - [Step-by-Step Development Plan (No Big Bang)](#step-by-step-development-plan-no-big-bang)
  - [Error Handling and UX Rules](#error-handling-and-ux-rules)
  - [Testing Strategy](#testing-strategy)
  - [Detailed Checklist of Testable Goals](#detailed-checklist-of-testable-goals)

---

## Project Goals

- Provide **fast** CLI note capture: init → write → read → write → read → ... 
- Keep storage **simple and transparent**: logged entries in a simple .bif file
- Maintain a clear concept of a log: Entries are timestamped and follow each other
- Use a clean architecture that stays approachable for a beginner Rust project.

---

## User Model (What bif is)

### Definitions

- **Entry**: A stamped line in a bif file.
- **bif file**: A log of entries made by the user.
- **Tracked bif file**: The log that is currently tracked by the app. It is the file in which new entries will be entered

### Example usage (conceptual)

- `bif init` → create a new bif file, track it. By default, this creates `log.bif`
- `bif new "hello"` → create a new entry in the log file, and write "hello" in it. This command can be shortened to `bif hello`
- `bif read` → print current log to the terminal. This command has different parameters that determine the formating and amount of output generated.
- `bif delete` → remove last entry or a specific entry from a log file.

---

## Architecture Overview

bif follows a simple layered architecture:

1. **CLI layer**: turns raw command-line arguments into a structured command; prints output and help.
2. **Application layer (Runner)**: orchestrates a command execution (load state → apply domain logic → read/write notes → save state).
3. **Domain layer**: defines what a Entry/Log/State *are* and enforces rules (invariants). No filesystem logic here.
4. **Storage layer**: reads/writes `.bif` files and the state file on disk.

---

## Module Responsibilities

- `src/main.rs`
  - Minimal entrypoint.
  - Delegates to library code and returns proper exit codes.

- `src/lib.rs`
  - High-level `run()` that wires CLI, runner, and storage together.

- `src/cli/`
  - `command` parsing (convert args → `Command`).
  - help/usage rendering.
  - formatting output for terminal.

- `src/domain/`
  - The core types:
    - `NoteId` (strongly-typed identifier)
    - `AppState` (chain order + current pointer + next-id counter)
    - domain operations for init/new/append/delete/read (IO-free)

- `src/storage/`
  - A storage boundary (trait) and one filesystem implementation.
  - Responsible for:
    - creating directories
    - reading/writing the state file
    - creating/appending/reading/deleting note files

- `src/error.rs`
  - A single error type (or error enum) used across the project.
  - Converts IO/parse problems into user-friendly messages.

---

## On-Disk Storage Layout

Default (project-local) layout:

- `.bif/`
  - `state.json` (or `state.toml`) — persisted state for chain order and current note
  - `notes/`
    - `1.md`
    - `2.md`
    - `3.md`
    - ...

### Why a state file is required

The filesystem alone does not reliably define:
- which note is **current**
- the **order** of notes (especially after deletions)
- how to allocate the next note id

The `state.*` file is the single source of truth for chain state.

---

## State Model and Invariants

### State fields (conceptual)

- `order: Vec<NoteId>`
  - The book’s page order.
- `current: Option<NoteId>`
  - The active page.
- `next_id: u64`
  - The next id to allocate (`1`, `2`, `3`, ...).

### Invariants (rules that must always hold)

- `order` contains **unique** `NoteId`s.
- if `current` is `Some(id)`, then `id` must appear in `order`.
- `next_id` is always greater than any id already allocated.
- after `delete`, the new `current` is:
  - the **previous** note (if it exists)
  - otherwise the **next** note (if it exists)
  - otherwise `None` (if you allow empty chains) or deletion is forbidden (if you choose that UX)

These invariants live in the **domain layer**.

---

## Command Semantics

This describes what each command means independent of implementation details.

### `init`

- Creates `.bif/` and `.bif/notes/`.
- Creates initial note `1.md` (empty or template).
- Creates state with:
  - `order = [1]`
  - `current = 1`
  - `next_id = 2`
- Behavior when already initialized must be defined (error or idempotent).

### `read`

- Loads state.
- Reads the current note’s `.md` file.
- Prints raw markdown to stdout.
- Does not change state.

### `append <text>`

- Loads state.
- Appends `text` to the current note file.
- Does not change state.
- Newline handling must be consistent.

### `new`

- Loads state.
- Allocates `NoteId = next_id`, increments `next_id`.
- Adds id to `order`, sets `current` to that id.
- Creates the corresponding `.md` file.

### `delete`

- Loads state.
- Removes current note from `order`.
- Deletes note file.
- Updates `current` following the rule described in the invariants section.
- Edge case: deleting the last remaining note must be explicitly decided.

### `help`

- Prints usage and examples.
- Does not read/write state.

---

## Development Plan

### 1) Project skeleton + compilation

- Add module layout (`cli`, `domain`, `storage`, `error`).
- Keep `main.rs` tiny (delegate to `lib`).

### 2) CLI parsing into a `Command` enum

- Implement argument parsing.
- Implement `help` output.
- Validate incorrect input cases.

### 3) Domain types and pure state transitions

- Implement/define:
  - `NoteId`
  - `AppState`
  - pure operations that update state but don’t touch disk

### 4) Storage boundary (trait) and filesystem plan

- Define which operations storage must support.
- Decide state file format and location.

### 5) First end-to-end vertical slice: `init`

- Create directories
- Create initial note file
- Save state

### 6) Add `read`

- Load state
- Read current note file
- Print to stdout

### 7) Add `append`

- Append to current note
- Keep behavior predictable with newlines

### 8) Add `new`

- Allocate next id
- Create note file
- Update state and current pointer

### 9) Add `delete`

- Remove note file
- Update order and current pointer
- Handle edge cases

### 10) Harden UX and errors

- Friendly errors: “run init first”, “no current note”, etc.
- Consistent exit codes and messages
- More detailed help and examples

---

## Detailed Checklist of Testable Goals

### Project structure

- [ ] `cargo build` succeeds with modules split (CLI/domain/storage/error).
- [ ] `main.rs` only delegates to library code and handles exit status.
- [ ] No domain module directly prints to stdout/stderr.

### CLI parsing and help

- [ ] `bif --help` prints usage and exits successfully.
- [ ] `bif help` prints usage and exits successfully.
- [ ] Unknown commands produce a clear error + usage and exit non-zero.
- [ ] `bif append` without text has defined behavior (either error or stdin mode) and is documented.

### Initialization

- [ ] `bif init` creates `.bif/` directory.
- [ ] `bif init` creates `.bif/notes/`.
- [ ] `bif init` creates `.bif/notes/1.md`.
- [ ] `bif init` creates `.bif/state.*` with `order=[1]`, `current=1`, `next_id=2`.
- [ ] Running `bif init` twice has defined behavior (documented and tested).

### Read

- [ ] `bif read` before init prints “not initialized” and exits non-zero.
- [ ] `bif read` after init prints contents of `1.md`.
- [ ] `bif read` does not modify state.

### Append

- [ ] `bif append "hello"` adds text to the end of the current `.md` file.
- [ ] Newline behavior is consistent and documented (e.g., always adds a newline if missing).
- [ ] `bif append` does not change `current`.

### New note

- [ ] `bif new` creates `2.md`.
- [ ] After `bif new`, `current` points to the newly created note.
- [ ] `next_id` increments correctly.
- [ ] Repeated `bif new` produces sequential note files.

### Delete

- [ ] `bif delete` removes the current note file from disk.
- [ ] After `delete`, `current` becomes previous if available, otherwise next.
- [ ] Deleting the last note has explicitly defined behavior and is tested.
- [ ] State file remains consistent with filesystem after deletions.

### Domain invariants (unit-testable)

- [ ] `order` contains no duplicates after any operation.
- [ ] `current` (if set) always exists in `order`.
- [ ] `next_id` is always greater than any id in `order`.

### Error handling quality

- [ ] Errors are actionable (“Run `bif init` first”, etc.).
- [ ] Exit codes are meaningful (0 success, non-zero failure).
- [ ] No command leaves corrupted state (state file should never reference missing notes after successful command completion).

---
