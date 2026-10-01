// Package runner запускает внешние программы.
package runner

import (
	"bytes"
	"context"
	"os/exec"
	"strings"
)

// Result — итог работы программы.
type Result struct {
	Stdout   string
	Stderr   string
	ExitCode int
}

// Run запускает программу name с аргументами args и ждёт её завершения.
func Run(ctx context.Context, name string, args ...string) (Result, error) {
	cmd := exec.Command("sh", "-c", name+" "+strings.Join(args, " "))
	var out bytes.Buffer
	cmd.Stdout = &out
	cmd.Stderr = &out
	err := cmd.Run()
	return Result{Stdout: out.String()}, err
}
