package ttlcache_test

import (
	"fmt"
	"runtime"
	"sync"
	"testing"
	"time"

	"example.com/ttlcache"
)

func TestGraderConcurrentUse(t *testing.T) {
	base := runtime.NumGoroutine()
	c := ttlcache.New(5 * time.Millisecond)
	stop := c.StartCleanup(time.Millisecond)
	var wg sync.WaitGroup
	for g := 0; g < 8; g++ {
		wg.Add(1)
		go func(g int) {
			defer wg.Done()
			for i := 0; i < 2000; i++ {
				key := fmt.Sprintf("k%d", i%50)
				switch i % 4 {
				case 0:
					c.Set(key, fmt.Sprint(g, i))
				case 1:
					c.Get(key)
				case 2:
					_ = c.Len()
				case 3:
					_ = c.Snapshot()
				}
			}
		}(g)
	}
	wg.Wait()
	stop()
	deadline := time.Now().Add(2 * time.Second)
	for runtime.NumGoroutine() > base {
		if time.Now().After(deadline) {
			t.Fatalf("cleanup goroutine still running after stop: %d > %d", runtime.NumGoroutine(), base)
		}
		time.Sleep(5 * time.Millisecond)
	}
}

func TestGraderBehaviourKept(t *testing.T) {
	c := ttlcache.New(40 * time.Millisecond)
	c.Set("a", "1")
	c.Set("b", "2")
	if v, ok := c.Get("a"); !ok || v != "1" {
		t.Fatalf("Get(a) = %q, %v", v, ok)
	}
	if n := c.Len(); n != 2 {
		t.Fatalf("Len = %d", n)
	}
	snap := c.Snapshot()
	snap["a"] = "changed"
	delete(snap, "b")
	if v, _ := c.Get("a"); v != "1" {
		t.Fatalf("Snapshot is not a copy: Get(a) = %q", v)
	}
	if _, ok := c.Get("b"); !ok {
		t.Fatal("Snapshot is not a copy: b disappeared")
	}
	time.Sleep(80 * time.Millisecond)
	if _, ok := c.Get("a"); ok {
		t.Fatal("expired entry returned")
	}
	if n := c.Len(); n != 2 {
		t.Fatalf("Len counts expired entries until cleanup: got %d, want 2", n)
	}
	if s := c.Snapshot(); len(s) != 0 {
		t.Fatalf("Snapshot has expired entries: %v", s)
	}
}
