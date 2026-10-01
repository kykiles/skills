// Package runner запускает внешние программы.
package runner

import (
	"bytes"
	"context"
	"errors"
	"os/exec"
)

// Result — итог работы программы.
type Result struct {
	Stdout   string
	Stderr   string
	ExitCode int
}

// Run запускает программу name с аргументами args без участия shell и ждёт её
// завершения. Аргументы передаются программе как есть.
//
// Ненулевой код выхода не считается ошибкой: он возвращается в Result.ExitCode.
// Ошибка возвращается, если программу не удалось запустить или она прервана;
// при отмене ctx программа останавливается, а для ошибки errors.Is(err, ctx.Err())
// истинно.
func Run(ctx context.Context, name string, args ...string) (Result, error) {
	cmd := exec.CommandContext(ctx, name, args...)
	var stdout, stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr

	err := cmd.Run()
	res := Result{Stdout: stdout.String(), Stderr: stderr.String()}
	var exitErr *exec.ExitError
	if errors.As(err, &exitErr) && exitErr.Exited() {
		res.ExitCode = exitErr.ExitCode()
		return res, nil
	}
	return res, err
}
