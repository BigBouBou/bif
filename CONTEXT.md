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
- `read` default must remain raw record output (script-friendly).
- `read --pretty` is a human view (presentation layer).
- Notes are 1-line records appended to a `.bif` file.
- Multiple `.bif` logs exist; exactly one is “tracked” (active target) at a time.
- Shortcut UX desired: `bif hello` == create a new entry with body `hello` (implemented: unknown command falls back to `new`).
- Stamps (VISION):
  - Stamps are created on `new` (capture-time) and stored per entry.
  - Stamps are consulted/rendered on `read` (view-time), primarily `read --pretty`.
  - User chooses stamp schema + rendering via a GLOBAL config file.
  - Each entry stores metadata including a config hash so `read --pretty` can assess compatibility.
  - Backward-compatible: pretty rendering must handle legacy entries without meta by falling back to canonical stamp fields or indicating missing stamps; raw mode always works.

## 3) Truth from code (what actually exists)
### Commands (CLI)
- Parsing in `src/cli/command.rs`.
- Commands enum:
  - `help`
  - `init [name]` (implemented)
  - `track <name>` (implemented; CWD-only)
  - `new <body...>` (implemented; unknown command falls back to `new`)
  - `delete [N|-N]` (implemented)
  - `read [N|-N] [--pretty]` (implemented)
    - Default `read` prints raw record lines unchanged (script-friendly)
    - `read --pretty` parses record lines and prints a presentation view
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
- Presentation-layer stamp formatting exists in `src/domain/stamp_format.rs`:
  - Types: `StampFormat { parts: Vec<StampPart> }`, `StampPart` enum
  - API: `render_stamp(&domain::entry::Stamp, &StampFormat) -> String`
  - `StampFormat::default_pretty()` used by CLI `read --pretty`
  - Timestamp rendering parses `Stamp.timestamp` as epoch seconds (UTC) when possible; if unparseable/negative, date/time parts fall back to the raw timestamp string.

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
- Tests added:
  - `src/domain/stamp_format.rs` unit tests for default pretty rendering + fallback behavior
  - `src/cli/command_tests.rs` unit tests for `read --pretty` parsing

### Stamp modularity / customization (NEXT)

#### High-level task plan (to reach user-selected, extensible, stored stamps)
1) Extend entry record with optional META field (backward-compatible)
   - Files: `bif/src/domain/entry.rs`
   - Change: accept 3-field legacy + 4-field meta format.
   - Decide META encoding + escaping rules; add `Entry.meta`.
   - Add tests: roundtrip 3-field legacy unchanged; 4-field parses/serializes; escape correctness.

2) Implement stamp providers executed on `new` (capture-time)
   - Files: new `bif/src/domain/stamp_provider.rs` (or `domain/stamps/`), update `bif/src/cli/command.rs`.
   - Built-in providers (start minimal): `time` (derived from epoch seconds), `date`, `datetime`, `level`, `source`, `cwd`.
   - Output: provider results stored into `Entry.meta` under stable keys (stamp IDs).

3) Implement pretty rendering that consumes stored META + checks config hash
   - Files: `bif/src/cli/command.rs`, `bif/src/domain/stamp_format.rs`.
   - Change `read --pretty` to primarily render using stored meta values.
   - Compatibility behavior:
     - If entry has no META (legacy): render using current `StampFormat` over canonical `Stamp` fields (best-effort).
     - If entry has META but `_cfg_hash` differs from current global config: show a clear warning or error (decision), and still allow raw `read`.
     - If META JSON invalid: treat as formatting error in pretty mode; raw `read` still works.

4) Add user selection of stamps (no config yet; CLI flag)
   - Files: `bif/src/cli/command.rs`
   - Add `bif new --stamps a,b,c "body"` and/or `bif read --pretty --stamps a,b,c`.
   - Default selection: a conservative set to avoid surprises; document in README later.

5) Add GLOBAL config file loading (required)
   - Files: new `bif/src/cli/config.rs` (or `domain/config.rs`), update `cli/command.rs`.
   - Config is global (user-level), not per-project CWD.
   - Config chooses:
     - which stamps to compute on `new` (capture-time providers)
     - how `read --pretty` renders them (ordering/layout)
   - Compute `_cfg_hash` from canonicalized config bytes; store into each entry META.

6) Add extensibility beyond built-ins
   - Template stamps: define stamp as format string over known fields/meta keys.
   - Optional command stamps: execute external command at `new` only (security risk; gate behind explicit opt-in).

#### Agent coordination prompts (copy/paste)

NOTE: run agents in order. Each agent must:
- read `bif/CONTEXT.md` first
- keep changes scoped to the files listed for that step
- run unit tests (or at least `cargo test`) and report failures
- avoid changing the raw `read` behavior

Step 0 — Coordinator prompt (use before delegating)
- "Read `bif/CONTEXT.md`. Produce an ordered implementation checklist for Steps 1–5 below. For each step, list files to edit, new types/APIs, and acceptance tests. Do NOT modify code."

Step 1 — Record + META_JSON (serde) foundation
- Agent 1 prompt:
  - "Read `bif/CONTEXT.md`. Implement META_JSON as an OPTIONAL 4th field in the entry record. Update `bif/src/domain/entry.rs` to parse both legacy 3-field and extended 4-field formats. Add `Entry.meta: BTreeMap<String,String>` (or equivalent). Use JSON for META (add `serde`/`serde_json` in `bif/Cargo.toml`). Ensure META is escaped/unescaped with the existing field escaping scheme. Add unit tests for: legacy roundtrip unchanged; meta roundtrip; invalid JSON error." 
  - Write-scope: `bif/src/domain/entry.rs`, `bif/Cargo.toml` (+ any new domain test module if needed).

Step 2 — Global config loading + hashing
- Agent 2 prompt:
  - "Read `bif/CONTEXT.md`. Implement GLOBAL config loading (user-level). Add `bif/src/cli/config.rs` (or `bif/src/domain/config.rs`) defining the config struct (stamps to compute on `new`, pretty layout/order). Implement canonicalization + hashing (stable bytes -> SHA-256 hex) to compute `_cfg_hash`. Add tests for hash stability (same config => same hash). Do not wire into commands yet." 
  - Write-scope: config module file(s) only (+ `bif/Cargo.toml` if adding hashing deps).

Step 3 — Stamp providers executed on `new` (capture-time)
- Agent 3 prompt:
  - "Read `bif/CONTEXT.md`. Implement a stamp provider registry executed on `bif new`. Create `bif/src/domain/stamp_provider.rs` (or `domain/stamps/`). Providers compute string values (e.g. `time`, `date`, `datetime`, `level`, `source`, `cwd`). Update `bif/src/cli/command.rs` `Command::NEW` to load global config, compute `_cfg_hash`, run configured providers, and store results into `Entry.meta` (including `_cfg_hash`). Ensure writing remains one-line safe via Entry serialization. Add unit tests for providers and for `new` populating meta." 
  - Write-scope: provider module + `bif/src/cli/command.rs`.

Step 4 — `read --pretty` consumes META + compatibility check
- Agent 4 prompt:
  - "Read `bif/CONTEXT.md`. Update `bif read --pretty` to prefer stored `Entry.meta` for rendering. Load global config and compute current `_cfg_hash`. For each entry: if meta missing => fallback to existing `domain::stamp_format` rendering from canonical `Stamp` (legacy). If meta present and `_cfg_hash` mismatches => print a clear warning (or error; follow the policy in CONTEXT) and still render best-effort or skip (make behavior explicit). If meta JSON invalid => pretty-mode formatting error. Keep raw `read` unchanged. Add tests for: legacy entry pretty; meta entry pretty; cfg mismatch behavior." 
  - Write-scope: `bif/src/cli/command.rs`, `bif/src/domain/stamp_format.rs` (if needed for meta rendering helpers).

Step 5 — Config-driven stamp selection + layout (no CLI flags)
- Agent 5 prompt:
  - "Read `bif/CONTEXT.md`. Wire global config into both `new` (which providers run) and `read --pretty` (which meta keys to display, and ordering/layout). If introducing a format DSL, keep it strict and test it heavily. Update README only if behavior changes visibly. Add integration-ish tests at CLI parse/execution layer if feasible." 
  - Write-scope: config module + `bif/src/cli/command.rs` + any new domain parsing/rendering modules.
- GOAL: allow user-selected, modular stamps.
  - Stamps are created at `new` time (capture-time) and stored in the record.
  - Stamps are consulted/rendered at `read` time (view-time), especially `read --pretty`.

- IMPORTANT CONSTRAINT: preserve existing record format compatibility.
  - Keep `Entry` line format stable: `<STAMP>\t<TAGS>\t<BODY>` with body escaping (see `domain/entry.rs`).
  - Keep existing `Stamp` record format parseable: `<TIMESTAMP>|<LEVEL>|<SOURCE?>`.
  - Add an extension mechanism for additional stamps without breaking old logs (see plan below).

- Current status (from code): presentation-layer stamp formatting exists (`domain/stamp_format.rs`) and `read --pretty` uses it.
  - This is currently NOT user-configurable and does NOT add new stored stamps.

- Status: FOUNDATION IMPLEMENTED (presentation-layer first)
  - `src/domain/stamp_format.rs` exists.
  - Enum-based parts (no `dyn`): `StampPart::{Literal, TimeHH, TimeMM, TimeSS, DateDD, DateMM, DateYYYY, Level, Source}`.
  - `render_stamp(&Stamp, &StampFormat) -> String` is the rendering API.
  - Timestamp interpretation: parse `Stamp.timestamp` as epoch seconds and render in UTC; if parse fails/negative, date/time parts fall back to the raw timestamp string.

- CLI surface area (minimal) now exists:
  - `bif read` default remains raw record output (unchanged).
  - `bif read --pretty` parses record lines with `Entry::from_record()` and prints: `<pretty_stamp>\t<body>`.

- NEXT ARCHITECTURE (VISION): extensible, stored stamps
  - Requirement (user decision): stamps must be computed on `new` and stored; `read` only consults what was stored.
  - Implication: need a record extension mechanism.

  - Proposed storage evolution (inference; implement next):
      - Keep the existing `STAMP` triple as-is for backward compatibility.
      - Add an OPTIONAL 4th field to the entry record for stamp metadata (JSON):
        - v1: `<STAMP>\t<TAGS>\t<BODY>\t<META_JSON?>`
        - where `META_JSON?` is optional and may be empty.
        - Parsing strategy: update `Entry::from_record()` to accept:
          - legacy 3-field lines: `<STAMP>\t<TAGS>\t<BODY>`
          - extended 4-field lines: `<STAMP>\t<TAGS>\t<BODY>\t<META_JSON>`

      - `META_JSON` rules (MUST be one-line safe):
        - Serialize compact JSON (no newlines).
        - Still escape/unescape the meta field using the same scheme as `BODY` (`escape_field`/`unescape_field`) to guarantee one-line records even if values contain tabs/newlines.
        - JSON shape: object with string keys. Values are strings.
        - Reserve `_`-prefixed keys for bif internals.

      - Required internal meta keys (computed on `new`):
        - `_cfg_hash`: hash of the global config that determined which stamps were computed + how.
        - (Optional) `_schema_version`: small integer for future migrations.

  - Domain model evolution (inference; implement next):
    - Extend `Entry` to include `meta: BTreeMap<String, String>` (or Vec of pairs) that stores stamp outputs created at capture time.
    - Validation: restrict keys (non-empty; no delimiters used by encoding) to keep on-disk parseable.

  - Stamp providers (capture-time):
    - Add a registry of built-in stamp providers executed on `new`.
    - Later allow user-defined providers via config (template stamps, command stamps).

- Config + customization DSL (DEFERRED; but required for user selection)
  - Not implemented: config loading (e.g. `.bif.toml` / XDG).
  - Not implemented: `--format` / stamp-selection DSL.
  - Next: introduce config that selects which stamps to compute on `new` and how to render on `read --pretty`.

### Existing functionality status (already DONE)
- `new <body>` behavior:
  - DONE: `new` creates `Entry` with `Stamp(timestamp=epoch seconds string, level=INFO, source=None)` then appends `Entry::to_record()` + `\n` to tracked file.
  - NOTE: `new/read/delete` hard-require a tracked log (CLI reads `.bif-tracked` and errors with guidance)

- Tracked log selection:
  - DONE (CWD dotfile): `src/storage/tracked.rs` persists `.bif-tracked`
  - DONE: `track <name>` validates name (no separators) + normalizes `.bif` + requires file exists in CWD
  - TODO: allow tracking by relative/absolute path? (decision)

- `read`:
  - DONE: `read` supports (RAW; unchanged):
    - `read` => print entire tracked file (raw)
    - `read 1` / `read -1` => print last record only
    - `read 2` => print last 2 records
    - `read -2` => print 2nd-to-last record only
  - DONE: `read --pretty` supports the same selectors but prints a presentation view:
    - Output: `<pretty_stamp>\t<body>` (stamp rendered by `domain::stamp_format`)
  - Storage strategy: read whole file or slice `.lines()` / helper fns; raw mode prints record line(s) exactly.

- `delete`:
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
