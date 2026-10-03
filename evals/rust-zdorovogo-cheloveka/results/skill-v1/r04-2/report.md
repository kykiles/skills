Refactoring of `format_report` function completed successfully.

## Changes:
- Created helper function `format_section` to eliminate 52 lines of duplicated code
- Refactored three nearly identical section handlers (Critical, Warning, Info) to use a single generic function with custom closures
- Function is now 8 lines shorter and much easier to maintain and modify

## Verification:
- Unit test passes: output is byte-for-byte identical
- cargo fmt: report.rs is properly formatted
- cargo clippy: no warnings
- cargo test: 1 test passed
- cargo check: success

File modified: /tmp/ws/skill-v1/r04-2/src/report.rs
The refactoring improves code clarity while preserving exact output byte-for-byte.
