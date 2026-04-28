# bif — Agent Context (for me)

## 0) Prime directive
- Optimize for *fast CLI capture* into plain text.
- Don’t over-engineer. Keep storage transparent + append-only where possible.
- Prefer changing code/CLI to match the domain record format already implemented in `domain::entry`.

## 1) What this repo is
- Rust CLI app: “Before I Forget” (`bif`).
- Goal: add notes quickly from terminal.
- Current implementation is partial.

## 2) Current user-facing shape (intended)
- Default log file: `log.bif` (or user-provided name).
- Notes are 1-line records appended to a `.bif` file.
- Multiple `.bif` logs exist; exactly one is “tracked” (active target) at a time.
- Shortcut UX desired: `bif hello` == create a new entry with body `hello`.

## 3) Truth from code (what actually exists)
### Commands (CLI)
- Parsing in `src/cli/command.rs`.
- Commands enum:
  - `help`
  - `init [name]` (implemented)
  - `track` (stub)
  - `new` (stub; body not wired)
  - `delete` (stub)
  - `read` (stub)
- `main.rs` prints `welcome()` then calls `run(args)`.
- `run()` parses `Command` then `execute()`.

### Storage (FS)
- `src/storage/fs_store.rs`:
  - `create_empty_record_file_in_cwd(file_name)` creates a NEW empty file in CWD.
  - Validates file name: non-empty, no path separators, must be a plain file name.
  - Uses `OpenOptions::create_new(true)` (no overwrite).
- No tracking/state persistence implemented yet.

### Domain model
- Real domain implementation exists in `src/domain/entry.rs`.

#### Record format (IMPORTANT)
- `Entry::to_record()` emits:
  - `<STAMP>\t<TAGS>\t<BODY>`
  - tags: comma-separated string (empty allowed)
  - body is escaped to stay one-line:
    - `\\` => `\\\\`
    - tab => `\\t`
    - newline => `\\n`
    - carriage return => `\\r`
- `Entry::from_record()` parses `splitn(3, '\t')` and validates.
- `Stamp` record format:
  - `<TIMESTAMP>|<LEVEL>|<SOURCE?>`
  - `SOURCE?` may be empty meaning `None`
  - `source` must not contain `|`.
- Entry invariants:
  - stamp valid
  - body non-empty (trim)
  - tags non-empty (trim) and must not contain `,`.

## 4) Immediate gaps / TODO map
- Implement actual `new <body>` behavior:
  - create Entry with Stamp (need timestamp source; decide format)
  - append record line to tracked `.bif` file
- Implement tracked log selection:
  - decide where to persist (e.g., simple dotfile like `.bif-tracked` in CWD or XDG)
  - `track` should set active log; `init` should also set it
- Implement `read`:
  - print tracked file contents (raw lines) or parse into entries (optional)
- Implement `delete`:
  - remove last entry (probably by rewriting file; keep simple)

## 5) Design constraints I should remember
- Keep logs as plain text `.bif` in the working directory (current `init` behavior).
- Avoid hidden complex folder layouts unless required.
- Failure modes should be actionable (e.g., “run `bif init` or `bif track` first”).
- Don’t print from domain layer; CLI renders messages.

## 6) Quick navigation
- Entry + record encoding/decoding: `bif/src/domain/entry.rs`
- CLI command parsing/execution: `bif/src/cli/command.rs`
- FS init helper: `bif/src/storage/fs_store.rs`
- Legacy error (planned removal): `bif/src/error.rs` (comment says LEGACY)
