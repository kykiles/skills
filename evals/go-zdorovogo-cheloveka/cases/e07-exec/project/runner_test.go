package runner

import (
	"context"
	"strings"
	"testing"
)

func TestRunEcho(t *testing.T) {
	res, err := Run(context.Background(), "echo", "hello")
	if err != nil {
		t.Fatal(err)
	}
	if strings.TrimSpace(res.Stdout) != "hello" {
		t.Fatalf("Stdout = %q", res.Stdout)
	}
}
