package batch_test

import (
	"context"
	"errors"
	"fmt"
	"runtime"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"example.com/batch"
)

type graderProbe struct {
	active, maxActive, calls atomic.Int64
}

func (p *graderProbe) enter() {
	p.calls.Add(1)
	n := p.active.Add(1)
	for {
		m := p.maxActive.Load()
		if n <= m || p.maxActive.CompareAndSwap(m, n) {
			return
		}
	}
}

func (p *graderProbe) leave() { p.active.Add(-1) }

func graderItems(n int) []string {
	items := make([]string, n)
	for i := range items {
		items[i] = fmt.Sprintf("item-%02d", i)
	}
	return items
}

// graderRun calls ProcessAll with a hang guard.
func graderRun(t *testing.T, ctx context.Context, items []string, workers int, fn func(context.Context, string) (int, error)) ([]batch.Result, error) {
	t.Helper()
	type ret struct {
		res []batch.Result
		err error
	}
	done := make(chan ret, 1)
	go func() {
		res, err := batch.ProcessAll(ctx, items, workers, fn)
		done <- ret{res, err}
	}()
	select {
	case r := <-done:
		return r.res, r.err
	case <-time.After(5 * time.Second):
		t.Fatal("ProcessAll hangs")
		return nil, nil
	}
}

func graderNoLeak(t *testing.T, base int) {
	t.Helper()
	deadline := time.Now().Add(2 * time.Second)
	for runtime.NumGoroutine() > base {
		if time.Now().After(deadline) {
			buf := make([]byte, 1<<16)
			n := runtime.Stack(buf, true)
			t.Fatalf("goroutines leaked: %d > %d\n%s", runtime.NumGoroutine(), base, buf[:n])
		}
		time.Sleep(10 * time.Millisecond)
	}
}

func graderNoCallsAfterReturn(t *testing.T, p *graderProbe) {
	t.Helper()
	if a := p.active.Load(); a != 0 {
		t.Fatalf("%d fn calls still running after ProcessAll returned", a)
	}
	calls := p.calls.Load()
	time.Sleep(100 * time.Millisecond)
	if c := p.calls.Load(); c != calls {
		t.Fatalf("fn called %d more times after ProcessAll returned", c-calls)
	}
}

func TestGraderSuccessOrderAndLimit(t *testing.T) {
	base := runtime.NumGoroutine()
	var p graderProbe
	items := graderItems(40)
	res, err := graderRun(t, context.Background(), items, 4, func(ctx context.Context, s string) (int, error) {
		p.enter()
		defer p.leave()
		time.Sleep(time.Millisecond)
		return len(s) * 2, nil
	})
	if err != nil {
		t.Fatal(err)
	}
	if len(res) != len(items) {
		t.Fatalf("len = %d", len(res))
	}
	for i, r := range res {
		if r.Item != items[i] || r.Value != len(items[i])*2 {
			t.Fatalf("res[%d] = %+v", i, r)
		}
	}
	if m := p.maxActive.Load(); m > 4 {
		t.Fatalf("max parallel calls = %d; want <= 4", m)
	}
	graderNoLeak(t, base)
}

var errGraderBoom = errors.New("boom")

func TestGraderErrorStopsEverything(t *testing.T) {
	base := runtime.NumGoroutine()
	var p graderProbe
	_, err := graderRun(t, context.Background(), graderItems(100), 4, func(ctx context.Context, s string) (int, error) {
		p.enter()
		defer p.leave()
		if s == "item-05" {
			return 0, errGraderBoom
		}
		// Ignores ctx on purpose: started calls must still be awaited.
		time.Sleep(20 * time.Millisecond)
		return 1, nil
	})
	if !errors.Is(err, errGraderBoom) {
		t.Fatalf("err = %v; want errGraderBoom", err)
	}
	graderNoCallsAfterReturn(t, &p)
	if c := p.calls.Load(); c > 40 {
		t.Fatalf("fn called %d times; new items should stop after the error", c)
	}
	graderNoLeak(t, base)
}

func TestGraderCancelStopsEverything(t *testing.T) {
	base := runtime.NumGoroutine()
	var p graderProbe
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	var once sync.Once
	_, err := graderRun(t, ctx, graderItems(100), 3, func(fctx context.Context, s string) (int, error) {
		p.enter()
		defer p.leave()
		if p.calls.Load() >= 6 {
			once.Do(cancel)
		}
		select {
		case <-fctx.Done():
			return 0, fctx.Err()
		case <-time.After(10 * time.Millisecond):
			return 1, nil
		}
	})
	if !errors.Is(err, context.Canceled) {
		t.Fatalf("err = %v; want context.Canceled", err)
	}
	graderNoCallsAfterReturn(t, &p)
	graderNoLeak(t, base)
}

func TestGraderCancelWithSlowCallsWaits(t *testing.T) {
	base := runtime.NumGoroutine()
	var p graderProbe
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Millisecond)
	defer cancel()
	_, err := graderRun(t, ctx, graderItems(50), 4, func(context.Context, string) (int, error) {
		p.enter()
		defer p.leave()
		time.Sleep(50 * time.Millisecond) // ignores ctx
		return 1, nil
	})
	if !errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("err = %v; want context.DeadlineExceeded", err)
	}
	graderNoCallsAfterReturn(t, &p)
	graderNoLeak(t, base)
}
