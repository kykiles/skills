package svcconfig

import (
	"os"
	"path/filepath"
	"testing"
)

func TestLoadBadJSON(t *testing.T) {
	path := filepath.Join(t.TempDir(), "config.json")
	if err := os.WriteFile(path, []byte("{"), 0o600); err != nil {
		t.Fatal(err)
	}
	cfg, err := Load(path)
	if err == nil || cfg != nil {
		t.Fatalf("Load() = %v, %v; want nil config and error", cfg, err)
	}
}
