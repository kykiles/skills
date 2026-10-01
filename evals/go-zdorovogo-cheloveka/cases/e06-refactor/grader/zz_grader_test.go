package report

// Grader: compares FormatReport with the original implementation byte for byte.

import (
	"fmt"
	"math/rand"
	"sort"
	"strings"
	"testing"
)

func zzGraderOriginalFormatReport(title string, entries []Entry) string {
	var b strings.Builder
	b.WriteString("=== " + title + " ===\n")

	critical := map[string][]Entry{}
	warning := map[string][]Entry{}
	info := map[string][]Entry{}
	for _, e := range entries {
		switch e.Level {
		case Critical:
			critical[e.Source] = append(critical[e.Source], e)
		case Warning:
			warning[e.Source] = append(warning[e.Source], e)
		case Info:
			info[e.Source] = append(info[e.Source], e)
		}
	}

	if len(critical) > 0 {
		b.WriteString("\n[" + Critical.String() + "]\n")
		var sources []string
		for s := range critical {
			sources = append(sources, s)
		}
		sort.Strings(sources)
		total := 0
		for _, s := range sources {
			n := 0
			var msgs []string
			seen := map[string]bool{}
			for _, e := range critical[s] {
				n += e.Count
				if !seen[e.Message] {
					seen[e.Message] = true
					msgs = append(msgs, e.Message)
				}
			}
			total += n
			b.WriteString(fmt.Sprintf("  %-12s %5d  %s\n", s, n, strings.Join(msgs, "; ")))
		}
		b.WriteString(fmt.Sprintf("  итого: %d\n", total))
	}

	if len(warning) > 0 {
		b.WriteString("\n[" + Warning.String() + "]\n")
		var sources []string
		for s := range warning {
			sources = append(sources, s)
		}
		sort.Strings(sources)
		total := 0
		for _, s := range sources {
			n := 0
			var msgs []string
			seen := map[string]bool{}
			for _, e := range warning[s] {
				n += e.Count
				if !seen[e.Message] {
					seen[e.Message] = true
					msgs = append(msgs, e.Message)
				}
			}
			total += n
			b.WriteString(fmt.Sprintf("  %-12s %5d  %s\n", s, n, strings.Join(msgs, "; ")))
		}
		b.WriteString(fmt.Sprintf("  итого: %d\n", total))
	}

	if len(info) > 0 {
		b.WriteString("\n[" + Info.String() + "]\n")
		var sources []string
		for s := range info {
			sources = append(sources, s)
		}
		sort.Strings(sources)
		total := 0
		for _, s := range sources {
			n := 0
			seen := map[string]bool{}
			for _, e := range info[s] {
				n += e.Count
				seen[e.Message] = true
			}
			total += n
			b.WriteString(fmt.Sprintf("  %-12s %5d  (%d сообщ.)\n", s, n, len(seen)))
		}
		b.WriteString(fmt.Sprintf("  итого: %d\n", total))
	}

	all := 0
	distinct := map[string]bool{}
	for _, e := range entries {
		all += e.Count
		distinct[e.Source] = true
	}
	b.WriteString(fmt.Sprintf("\nвсего событий: %d, источников: %d\n", all, len(distinct)))
	return b.String()
}

func zzGraderEntries(r *rand.Rand) []Entry {
	sources := []string{"db", "api", "очередь", "", "a-very-long-source-name", "API", "db "}
	messages := []string{"disk full", "timeout", "", "перезапуск", "timeout ", "x; y"}
	levels := []Level{Info, Warning, Critical, Critical, Level(3), Level(-1)}
	n := r.Intn(25)
	if r.Intn(10) == 0 {
		return nil
	}
	out := make([]Entry, n)
	for i := range out {
		out[i] = Entry{
			Source:  sources[r.Intn(len(sources))],
			Level:   levels[r.Intn(len(levels))],
			Message: messages[r.Intn(len(messages))],
			Count:   r.Intn(200000) - 1000,
		}
	}
	return out
}

func TestGraderSameOutputAsOriginal(t *testing.T) {
	r := rand.New(rand.NewSource(20261001))
	for i := 0; i < 3000; i++ {
		entries := zzGraderEntries(r)
		title := fmt.Sprintf("отчёт %d", i)
		before := append([]Entry(nil), entries...)
		want := zzGraderOriginalFormatReport(title, entries)
		got := FormatReport(title, entries)
		if got != want {
			t.Fatalf("case %d: output differs\nentries: %+v\ngot:\n%s\nwant:\n%s", i, entries, got, want)
		}
		for j := range entries {
			if entries[j] != before[j] {
				t.Fatalf("case %d: FormatReport modified its input", i)
			}
		}
	}
}

func TestGraderEdgeCases(t *testing.T) {
	cases := [][]Entry{
		nil,
		{},
		{{Source: "x", Level: Level(9), Message: "m", Count: 5}},
		{{Source: "s", Level: Info, Message: "a", Count: 1}, {Source: "s", Level: Info, Message: "a", Count: 1}},
		{{Source: "s", Level: Warning, Message: "b", Count: 1}, {Source: "s", Level: Warning, Message: "a", Count: 1}, {Source: "s", Level: Warning, Message: "b", Count: 1}},
	}
	for i, entries := range cases {
		if got, want := FormatReport("t", entries), zzGraderOriginalFormatReport("t", entries); got != want {
			t.Fatalf("edge %d: got\n%s\nwant\n%s", i, got, want)
		}
	}
}

var _ = sort.Strings
var _ = strings.Join
