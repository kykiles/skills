package logscan_test

import (
	"errors"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
	"syscall"
	"testing"

	"example.com/logscan"
)

func TestGraderManyFilesUnderFDLimit(t *testing.T) {
	dir := t.TempDir()
	var paths []string
	for i := 0; i < 400; i++ {
		p := filepath.Join(dir, fmt.Sprintf("f%03d.log", i))
		if err := os.WriteFile(p, []byte("x error\ny\n"), 0o600); err != nil {
			t.Fatal(err)
		}
		paths = append(paths, p)
	}
	var old syscall.Rlimit
	if err := syscall.Getrlimit(syscall.RLIMIT_NOFILE, &old); err != nil {
		t.Skip("no rlimit:", err)
	}
	lim := old
	lim.Cur = 128
	if err := syscall.Setrlimit(syscall.RLIMIT_NOFILE, &lim); err != nil {
		t.Skip("cannot lower rlimit:", err)
	}
	defer syscall.Setrlimit(syscall.RLIMIT_NOFILE, &old)

	got, err := logscan.CountMatches(paths, "error")
	if err != nil {
		t.Fatalf("CountMatches on 400 files with 128 fds: %v", err)
	}
	for _, p := range paths {
		if got[p] != 1 {
			t.Fatalf("%s: %d; want 1", p, got[p])
		}
	}
}

func TestGraderLongLines(t *testing.T) {
	dir := t.TempDir()
	p := filepath.Join(dir, "big.log")
	long := strings.Repeat("a", 300<<10) + " error " + strings.Repeat("b", 10)
	body := "error first\n" + long + "\nmiddle\n" + long + "\nerror last\n"
	if err := os.WriteFile(p, []byte(body), 0o600); err != nil {
		t.Fatal(err)
	}
	got, err := logscan.CountMatches([]string{p}, "error")
	if err != nil {
		t.Fatalf("CountMatches with 300 KiB lines: %v", err)
	}
	if got[p] != 4 {
		t.Fatalf("count = %d; want 4", got[p])
	}
}

func TestGraderNoTrailingNewlineAndCRLF(t *testing.T) {
	dir := t.TempDir()
	p := filepath.Join(dir, "crlf.log")
	if err := os.WriteFile(p, []byte("error a\r\nok\r\nerror b"), 0o600); err != nil {
		t.Fatal(err)
	}
	got, err := logscan.CountMatches([]string{p}, "error")
	if err != nil || got[p] != 2 {
		t.Fatalf("CountMatches = %v, %v; want 2", got, err)
	}
}

func TestGraderMissingFile(t *testing.T) {
	dir := t.TempDir()
	ok := filepath.Join(dir, "ok.log")
	os.WriteFile(ok, []byte("error\n"), 0o600)
	got, err := logscan.CountMatches([]string{ok, filepath.Join(dir, "absent.log")}, "error")
	if !errors.Is(err, fs.ErrNotExist) || got != nil {
		t.Fatalf("CountMatches = %v, %v; want nil and fs.ErrNotExist", got, err)
	}
}
