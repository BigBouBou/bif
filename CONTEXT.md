# bif — Agent Context (for me)

## HOW TO MAINTAIN THIS FILE (Agent Prompt)
You are updating `CONTEXT.md` for your future self (another instance of you). Follow this procedure strictly:

1) Source of truth order
- Prefer facts from code in `bif/src/**` over docs, comments, or assumptions.
- If you infer something, label it explicitly as an inference.

2) Update workflow
- Before editing, scan for changes in:
  - `bif/src/cli/**` (commands, parsing, UX)
  - `bif/src/domain/**` (record format, invariants)
  - `bif/src/storage/**` (on-disk layout, tracking state)
  - `bif/README.md` (user-facing claims)
- Then update the sections below:
  - "Truth from code" first (what exists)
  - then "Intended UX" (what we want)
  - then "Docs mismatch note" (differences + actions)
  - then "Immediate gaps / TODO map" (next steps)

3) Writing rules (optimize for agent memory)
- Write in English, for yourself, not for humans.
- Use short bullets, imperative voice, and stable headings.
- Keep it brutally specific: file paths, function names, formats, invariants.
- Record any non-obvious decisions (why, trade-offs, constraints).
- Avoid narrative. Avoid marketing language.

4) Integrity checks
- If you mention a format (e.g., record layout), verify it in code and include the exact delimiter/escaping rules.
- If README contradicts code, note it and propose the single canonical format.
- If a module is marked legacy, keep a note until it is deleted.

5) What not to include
- No personal fluff, no user instructions, no broad Rust tutorials.
- No guesses presented as facts.

(End of maintenance prompt)

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
- Shortcut UX desired: `bif hello` == create a new entry with body `hello` (implemented: unknown command falls back to `new`).

## 3) Truth from code (what actually exists)
### Commands (CLI)
- Parsing in `src/cli/command.rs`.
- Commands enum:
  - `help`
  - `init [name]` (implemented)
  - `track <name>` (implemented; CWD-only)
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
- `src/storage/tracked.rs`:
  - Tracking state persisted in CWD dotfile `.bif-tracked`.
  - `set_tracked_file_path(path: &str) -> io::Result<()>` writes `"<path>\n"`.
  - `get_tracked_file_path() -> io::Result<String>` reads first line + trims.
  - `tracked_file_path() -> io::Result<PathBuf>` returns `<CWD>/.bif-tracked`.

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
  - DONE: `new` creates `Entry` with `Stamp(timestamp=epoch seconds string, level=INFO, source=None)` then appends `Entry::to_record()` + `\n` to tracked file.
  - NOTE: `new/read/delete` hard-require a tracked log (CLI reads `.bif-tracked` and errors with guidance)
- Implement tracked log selection:
  - DONE (CWD dotfile): `src/storage/tracked.rs` persists `.bif-tracked`
  - DONE: `track <name>` validates name (no separators) + normalizes `.bif` + requires file exists in CWD
  - TODO: allow tracking by relative/absolute path? (decision)
  - TODO: error message when no tracked file for `new/read/delete`
- Implement `read`:
  - DONE: `read` supports:
    - `read` => print entire tracked file (raw)
    - `read 1` / `read -1` => print last record only
    - `read 2` => print last 2 records
    - `read -2` => print 2nd-to-last record only
  - Storage strategy: read whole file, slice `.lines()`, print raw record line(s).
- Implement `delete`:
  - DONE: `delete` supports:
    - `delete` / `delete 1` / `delete -1` => delete last record
    - `delete 2` => delete last 2 records
    - `delete -2` => delete 2nd-to-last record
  - Storage strategy: read whole file, manipulate `.lines()`, rewrite file (errors on empty/out-of-range).

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
