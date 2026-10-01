package cmdargs

import (
	"slices"
	"testing"
)

func TestBuild(t *testing.T) {
	got := Build([]string{"-v"}, "run", "x")
	want := []string{"-v", "run", "x"}
	if !slices.Equal(got, want) {
		t.Fatalf("Build() = %q; want %q", got, want)
	}
}

func TestWithoutEmpty(t *testing.T) {
	got := WithoutEmpty([]string{"a", "", "b"})
	want := []string{"a", "b"}
	if !slices.Equal(got, want) {
		t.Fatalf("WithoutEmpty() = %q; want %q", got, want)
	}
}
