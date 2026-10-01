package report

import "testing"

func TestFormatReport(t *testing.T) {
	got := FormatReport("сутки", []Entry{
		{Source: "db", Level: Critical, Message: "disk full", Count: 2},
		{Source: "api", Level: Info, Message: "restart", Count: 1},
	})
	want := `=== сутки ===

[Critical]
  db               2  disk full
  итого: 2

[Info]
  api              1  (1 сообщ.)
  итого: 1

всего событий: 3, источников: 2
`
	if got != want {
		t.Fatalf("FormatReport mismatch:\n%s\nwant:\n%s", got, want)
	}
}
