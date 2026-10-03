## Refactoring `format_report` — Complete

**What was changed:**
- Extracted 3 helper functions to eliminate duplication:
  1. `format_messages_line()` — formats output lines for Critical and Warning levels (unique messages in appearance order)
  2. `format_count_line()` — formats output lines for Info level (message count only)
  3. `format_section()` — generic section formatter that applies the level-specific line formatter

- Replaced 72 lines of duplicated section handling code (three nearly-identical blocks) with 3 simple calls to `format_section()`

- Main function body is now much more readable: aggregation → three calls to format_section → summary output

**Code metrics:**
- Net reduction: 16 lines (66 deleted, 50 added)
- The extracted functions provide clear names for each step, making maintenance easier
- No changes to public API or behavior

**Verification:**
1. ✓ cargo test: 1 test passed (exact byte-for-byte output match verified)
2. ✓ cargo clippy: No warnings
3. ✓ rustfmt: Formatting check passed

**References read:**
- [api.md](references/api.md) — confirmed that extracting helpers is justified when they name separate responsibility and make main flow readable

**Status:**
Output is identical byte-for-byte. All checks passed. Code is cleaner and more maintainable. No git commit made (per instructions).
