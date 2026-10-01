Fixed two critical issues in ProcessAll function in batch package:

ISSUES FIXED:
1. Goroutine leak: Workers were blocked forever sending results when ProcessAll returned early on error/cancellation
2. Race condition: Calls continued executing after ProcessAll returned, attempting to write to closed connections

ROOT CAUSE:
ProcessAll returned early without draining the outcomes channel, leaving workers blocked and violating its contract.

CHANGES:
File: batch.go
- Modified main loop to track iteration index (changed `for range items` to `for i := 0; i < len(items); i++`)
- Added outcome draining before returning on error: drain remaining (i+1) to len(items)-1 outcomes
- Added outcome draining before returning on cancellation: drain remaining i to len(items)-1 outcomes
- This ensures all worker goroutines complete before ProcessAll returns

File: batch_test.go
- Added TestProcessAllErrorStopsProcessing: verifies error handling
- Added TestProcessAllNoGoroutineLeakOnError: confirms no goroutine leak on error
- Added TestProcessAllNoGoroutineLeakOnCancel: confirms no goroutine leak on context cancellation
- All tests use sync/atomic for thread-safe concurrent access

VERIFICATION RESULTS:
✅ go test: 4/4 tests pass (0.604s)
✅ go test -race: 4/4 tests pass (1.614s) - no data races
✅ go vet: passes without warnings
✅ Contract maintained: function behavior unchanged for callers

The fix is minimal (12 lines changed in batch.go, 108 lines of comprehensive tests) and fully backward compatible.
