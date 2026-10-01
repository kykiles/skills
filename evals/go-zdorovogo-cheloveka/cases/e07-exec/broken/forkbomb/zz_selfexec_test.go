package runner

import (
	"context"
	"os"
	"testing"
)

// Плохо: дочерний процесс не узнаёт свой режим и снова запускает этот тест.
func TestSelfExecWithoutHelperMode(t *testing.T) {
	if _, err := Run(context.Background(), os.Args[0], "-test.run=TestSelfExecWithoutHelperMode"); err != nil {
		t.Fatal(err)
	}
}
