package runner_test

import (
	"context"
	"encoding/json"
	"errors"
	"os/exec"
	"path/filepath"
	"runtime"
	"slices"
	"strings"
	"sync"
	"testing"
	"time"

	"example.com/runner"
)

var (
	graderOnce   sync.Once
	graderHelper string
	graderErr    error
)

func graderHelperPath(t *testing.T) string {
	t.Helper()
	graderOnce.Do(func() {
		dir, err := filepathTemp()
		if err != nil {
			graderErr = err
			return
		}
		name := "helper"
		if runtime.GOOS == "windows" {
			name += ".exe"
		}
		graderHelper = filepath.Join(dir, name)
		out, err := exec.Command("go", "build", "-o", graderHelper, "./zz_graderhelper").CombinedOutput()
		if err != nil {
			graderErr = errors.New(string(out))
		}
	})
	if graderErr != nil {
		t.Fatal(graderErr)
	}
	return graderHelper
}

func TestGraderArgsVerbatim(t *testing.T) {
	args := []string{"a b", `"quoted"`, "$HOME", "x;y", "*", "", "--flag=1 2", "it's", `back\slash`}
	res, err := runner.Run(context.Background(), graderHelperPath(t), append([]string{"echo"}, args...)...)
	if err != nil {
		t.Fatalf("Run: %v", err)
	}
	var got []string
	if err := json.Unmarshal([]byte(res.Stdout), &got); err != nil {
		t.Fatalf("Stdout %q is not the child's JSON: %v", res.Stdout, err)
	}
	if !slices.Equal(got, args) {
		t.Fatalf("child got args %q; want %q", got, args)
	}
	if res.Stderr != "to-stderr" || res.ExitCode != 0 {
		t.Fatalf("Stderr = %q, ExitCode = %d", res.Stderr, res.ExitCode)
	}
}

func TestGraderNonZeroExitIsNotError(t *testing.T) {
	res, err := runner.Run(context.Background(), graderHelperPath(t), "exit", "3")
	if err != nil {
		t.Fatalf("Run returned error for non-zero exit: %v", err)
	}
	if res.ExitCode != 3 || !strings.Contains(res.Stderr, "boom") || res.Stdout != "" {
		t.Fatalf("Result = %+v; want ExitCode 3, Stderr boom, empty Stdout", res)
	}
}

func TestGraderMissingProgram(t *testing.T) {
	if _, err := runner.Run(context.Background(), "zz-no-such-program-zz", "x"); err == nil {
		t.Fatal("Run of a missing program returned nil error")
	}
}

func TestGraderDeadlineStopsChild(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), 200*time.Millisecond)
	defer cancel()
	start := time.Now()
	_, err := runner.Run(ctx, graderHelperPath(t), "sleep", "10s")
	if d := time.Since(start); d > 3*time.Second {
		t.Fatalf("Run ignored ctx: took %v", d)
	}
	if !errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("err = %v; want errors.Is(err, context.DeadlineExceeded)", err)
	}
}

func TestGraderCanceledBeforeStart(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	if _, err := runner.Run(ctx, graderHelperPath(t), "echo"); !errors.Is(err, context.Canceled) {
		t.Fatalf("err = %v; want errors.Is(err, context.Canceled)", err)
	}
}
