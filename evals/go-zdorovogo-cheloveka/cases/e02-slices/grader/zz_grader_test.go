package cmdargs_test

import (
	"slices"
	"testing"

	"example.com/cmdargs"
)

func TestGraderBuildDoesNotShareSpareCapacity(t *testing.T) {
	base := make([]string, 2, 10)
	base[0], base[1] = "-a", "-b"
	first := cmdargs.Build(base, "one")
	second := cmdargs.Build(base, "two")
	if !slices.Equal(first, []string{"-a", "-b", "one"}) {
		t.Fatalf("first = %q after second Build; want [-a -b one]", first)
	}
	if !slices.Equal(second, []string{"-a", "-b", "two"}) {
		t.Fatalf("second = %q", second)
	}
	if !slices.Equal(base, []string{"-a", "-b"}) {
		t.Fatalf("base changed: %q", base)
	}
}

func TestGraderWithoutEmptyKeepsInput(t *testing.T) {
	in := []string{"", "a", "", "b", "c", ""}
	orig := slices.Clone(in)
	got := cmdargs.WithoutEmpty(in)
	if !slices.Equal(got, []string{"a", "b", "c"}) {
		t.Fatalf("WithoutEmpty = %q", got)
	}
	if !slices.Equal(in, orig) {
		t.Fatalf("input changed: %q; want %q", in, orig)
	}
}

func TestGraderProfileCommandsAreIndependent(t *testing.T) {
	p := cmdargs.NewProfile("--retries=3", "", "--fast")
	run := p.Command("run", "job-1")
	stop := p.Command("stop", "job-2")
	if !slices.Equal(run, []string{"--quiet", "--color=never", "--retries=3", "--fast", "run", "job-1"}) {
		t.Fatalf("run = %q", run)
	}
	if !slices.Equal(stop, []string{"--quiet", "--color=never", "--retries=3", "--fast", "stop", "job-2"}) {
		t.Fatalf("stop = %q", stop)
	}
	again := p.Command("", "status")
	if !slices.Equal(again, []string{"--quiet", "--color=never", "--retries=3", "--fast", "status"}) {
		t.Fatalf("again = %q", again)
	}
	if !slices.Equal(run, []string{"--quiet", "--color=never", "--retries=3", "--fast", "run", "job-1"}) {
		t.Fatalf("run changed after later commands: %q", run)
	}
}
