# headercheck

Ensures every source file in a directory has the correct license/copyright
header, and can insert the missing one with `--fix`. This is distinct
from this workspace's `licenselint`, which audits the **licenses of your
dependencies** (what `Cargo.lock`/`package.json` pull in); `headercheck`
audits **your own source files** — the "every file needs a copyright
banner" policy some companies and OSS projects enforce, usually checked
today with a hand-rolled shell script or not checked at all.

## Usage

```bash
headercheck check src/ --ext rs --holder "Acme Corp"
headercheck fix   src/ --ext rs --holder "Acme Corp"

# or supply the exact header text yourself, including a {year} placeholder:
headercheck check src/ --ext rs --ext ts --template header.txt
```

`--ext` is repeatable (check multiple file types in one run). Give
either `--holder <name>` (builds `// Copyright (c) {year} <name>`,
`//` overridable via `--prefix`) or `--template <file>` (the file's
exact content becomes the header, `{year}` substituted where present) —
not both. Exit code `1` from `check` if anything is missing.

## How year matching works

A header is inserted with the **current** year, but `check` accepts
**any** 4-digit year in that position — a file headered in 2019 still
passes today. `{year}` must match exactly 4 ASCII digits, no more, no
fewer, so a header reading "20264" (a typo, or a different field
entirely) is correctly rejected rather than loosely accepted.

## Shebang handling

If a file starts with a real shebang line (`#!/usr/bin/env python3`),
the header goes **after** it, not before — a header inserted above a
shebang would break the file's ability to actually run as a script.
`check` and `fix` both understand this.

## Status: built and verified against real files on disk, including a real check → fix → re-check cycle

- **17 unit tests** (`cargo test --lib`): template rendering and
  matching (exact match, an older year still matching, a malformed
  non-4-digit year rejected, a missing header rejected, a wrong holder
  name rejected, a template with two independent `{year}` placeholders
  both validated), shebang-aware header placement (inserted after a
  real leading shebang line, not before), and file discovery (extension
  filtering, skipping `.git`/`target`/`node_modules`/etc., recursing
  into nested directories).
- **A real test-design mistake caught and removed, not silently left
  in**: an early test asserted a header appearing *before* a
  shebang-shaped line should fail to match. On reflection (and
  confirmed by actually running it — it failed) this doesn't correspond
  to a real scenario: a shebang only functions as one when it's the
  file's literal first line, so if a header genuinely comes first in the
  file's bytes, the file doesn't have a working shebang in the first
  place, and matching the header at position 0 is the correct behavior,
  not a bug. Removed the test rather than "fixing" working code to
  satisfy an invalid premise.
- **Live-verified end-to-end through the actual compiled binary**: a
  real scratch directory with one file already carrying a valid
  header stamped with an old year (2020) and one file with no header at
  all. `check` correctly flagged only the missing one and left the
  old-but-valid one alone (exit 1). `fix` inserted a real header
  (rendered with the real current year) into exactly the missing file,
  with a blank line separating it from the original content. Running
  `check` again afterward reported both files clean (exit 0) — a real
  check → fix → re-check cycle, not just each command tested in
  isolation.

**Not done / deliberately deferred**: per-file-type comment syntax
(`--prefix` is one fixed string for the whole run — a mixed Rust/Python
tree needs two separate invocations, one per `--prefix`/`--ext`
combination, rather than auto-detecting `#` vs `//` per extension);
multi-line header templates with embedded blank lines in the *check*
path work fine (the whole template file is matched verbatim aside from
`{year}`), but there's no support for a header block that should
tolerate reformatting/rewrapping; and no `--exclude` glob for skipping
specific files beyond the fixed VCS/build directory skip list.
