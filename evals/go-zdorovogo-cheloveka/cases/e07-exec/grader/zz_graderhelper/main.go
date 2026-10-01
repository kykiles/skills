// Command zz_graderhelper is a child process for the grader.
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"time"
)

func main() {
	switch os.Args[1] {
	case "echo":
		json.NewEncoder(os.Stdout).Encode(os.Args[2:])
		fmt.Fprint(os.Stderr, "to-stderr")
	case "exit":
		fmt.Fprint(os.Stderr, "boom")
		code, _ := strconv.Atoi(os.Args[2])
		os.Exit(code)
	case "sleep":
		d, _ := time.ParseDuration(os.Args[2])
		time.Sleep(d)
	}
}
