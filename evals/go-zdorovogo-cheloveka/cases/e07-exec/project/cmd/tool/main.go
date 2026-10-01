// Команда tool запускает программу и печатает её вывод.
package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"

	"example.com/runner"
)

func main() {
	if len(os.Args) < 2 {
		fmt.Fprintln(os.Stderr, "usage: tool <program> [args...]")
		os.Exit(2)
	}
	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt)
	defer stop()

	res, err := runner.Run(ctx, os.Args[1], os.Args[2:]...)
	if err != nil {
		fmt.Fprintln(os.Stderr, "tool:", err)
		os.Exit(1)
	}
	fmt.Print(res.Stdout)
	fmt.Fprint(os.Stderr, res.Stderr)
	os.Exit(res.ExitCode)
}
