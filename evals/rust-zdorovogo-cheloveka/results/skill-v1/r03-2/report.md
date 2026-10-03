## Summary of Changes

Successfully added "Returned" order status with all required rules and validations.

### Files Modified: 3
- `/tmp/ws/skill-v1/r03-2/src/status.rs` — Added Status::Returned enum variant, updated code/from_code/title methods
- `/tmp/ws/skill-v1/r03-2/src/policy.rs` — Updated business rules: is_open(), can_move(), refund()
- `/tmp/ws/skill-v1/r03-2/src/report.rs` — Updated test expectations for summary output

### Implementation Details

**Status Definition (status.rs)**:
- Added `Status::Returned` variant with comment "Возвращён после доставки" (Returned after delivery)
- Export code: `"returned"`
- Summary title: `"возвращён"`
- Updated `Status::ALL` array from 5 to 6 statuses

**Business Rules (policy.rs)**:
- `is_open()`: Returned orders are closed (cannot transition further)
- `can_move()`: Only transition allowed is Delivered → Returned
- `refund()`: Returned orders receive full refund (same as Cancelled)
- `can_cancel()`: Returned orders cannot be cancelled

**Summary Reporting (report.rs)**:
- Returned orders appear in their own row in summary output
- Returned orders excluded from revenue calculation
- Returned orders included in refund totals

### Test Results
✓ All 5 tests passed (3 original + 2 new comprehensive tests):
  - `status::tests::codes_round_trip` — Validates round-trip encoding
  - `policy::tests::lifecycle` — Validates state transitions
  - `policy::tests::returned_status` — NEW: Tests returned-specific rules
  - `report::tests::summary_counts` — Updated for new status in output
  - `report::tests::returned_orders_excluded_from_revenue` — NEW: Validates revenue exclusion

### Verification Completed
✓ `cargo build --offline` — successful
✓ `cargo test --offline` — 5/5 tests passed
✓ `cargo fmt --check` — formatting valid
✓ `cargo clippy` — no warnings or errors

All requirements met: only delivered orders can be returned, no further transitions possible, no cancellation allowed, full refund issued, excluded from revenue, separate summary line.
