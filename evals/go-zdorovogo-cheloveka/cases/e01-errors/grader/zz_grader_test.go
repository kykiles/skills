package svcconfig_test

import (
	"errors"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"example.com/svcconfig"
)

func graderWrite(t *testing.T, body string) string {
	t.Helper()
	path := filepath.Join(t.TempDir(), "config.json")
	if err := os.WriteFile(path, []byte(body), 0o600); err != nil {
		t.Fatal(err)
	}
	return path
}

func TestGraderValidConfigHasNilError(t *testing.T) {
	path := graderWrite(t, `{"port": 9000, "log_level": "debug"}`)
	cfg, err := svcconfig.Load(path)
	if err != nil {
		t.Fatalf("Load(valid) error = %#v (%T); want untyped nil", err, err)
	}
	if cfg == nil || cfg.Port != 9000 || cfg.LogLevel != "debug" || cfg.Addr != "127.0.0.1" {
		t.Fatalf("Load(valid) = %+v; want port 9000, debug, default addr", cfg)
	}
}

func TestGraderMissingFileIsNotExist(t *testing.T) {
	path := filepath.Join(t.TempDir(), "absent.json")
	cfg, err := svcconfig.Load(path)
	if !errors.Is(err, fs.ErrNotExist) {
		t.Fatalf("errors.Is(err, fs.ErrNotExist) = false; err = %v", err)
	}
	if cfg != nil {
		t.Fatalf("cfg = %+v; want nil on error", cfg)
	}
	if !strings.Contains(err.Error(), path) {
		t.Fatalf("error %q lost the file path", err)
	}
}

func TestGraderValidationErrorIsDistinguishable(t *testing.T) {
	for _, tc := range []struct{ body, field string }{
		{`{"port": 0}`, "port"},
		{`{"port": 80, "log_level": "loud"}`, "log_level"},
	} {
		cfg, err := svcconfig.Load(graderWrite(t, tc.body))
		var verr *svcconfig.ValidationError
		if !errors.As(err, &verr) || verr == nil {
			t.Fatalf("%s: errors.As(*ValidationError) failed; err = %v", tc.body, err)
		}
		if verr.Field != tc.field {
			t.Fatalf("%s: Field = %q; want %q", tc.body, verr.Field, tc.field)
		}
		if cfg != nil {
			t.Fatalf("%s: cfg = %+v; want nil on error (documented contract)", tc.body, cfg)
		}
	}
}

func TestGraderBadJSON(t *testing.T) {
	cfg, err := svcconfig.Load(graderWrite(t, `{"port": "x"`))
	if err == nil || cfg != nil {
		t.Fatalf("Load(bad json) = %v, %v; want nil, error", cfg, err)
	}
	if errors.Is(err, fs.ErrNotExist) {
		t.Fatalf("bad JSON reported as missing file: %v", err)
	}
}
