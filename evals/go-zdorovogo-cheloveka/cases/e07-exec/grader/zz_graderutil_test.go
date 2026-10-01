package runner_test

import "os"

func filepathTemp() (string, error) { return os.MkdirTemp("", "grader-helper-") }
