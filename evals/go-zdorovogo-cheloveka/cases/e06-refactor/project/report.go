// Package report формирует текстовый отчёт по событиям мониторинга.
package report

import (
	"fmt"
	"sort"
	"strings"
)

// Entry — агрегированное событие мониторинга.
type Entry struct {
	Source  string
	Level   Level
	Message string
	Count   int
}

// FormatReport возвращает отчёт: секции по уровням от Critical к Info, внутри
// секции источники по алфавиту с суммой Count. Для Critical и Warning выводятся
// уникальные сообщения в порядке появления, для Info — только их число.
// События с неизвестным уровнем в секции не попадают, но учитываются в итоге.
func FormatReport(title string, entries []Entry) string {
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
