package logscan

import (
	"os"
	"path/filepath"
	"testing"
)

func TestCountMatches(t *testing.T) {
	dir := t.TempDir()
	p := filepath.Join(dir, "a.log")
	if err := os.WriteFile(p, []byte("error one\nok\nerror two\n"), 0o600); err != nil {
		t.Fatal(err)
	}
	got, err := CountMatches([]string{p}, "error")
	if err != nil {
		t.Fatal(err)
	}
	if got[p] != 2 {
		t.Fatalf("CountMatches = %v; want 2", got)
	}
}
