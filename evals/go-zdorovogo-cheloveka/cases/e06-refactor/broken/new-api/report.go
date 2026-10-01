// Package report формирует текстовый отчёт по событиям мониторинга.
package report

import (
	"fmt"
	"maps"
	"slices"
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

	bySource := map[Level]map[string][]Entry{}
	for _, e := range entries {
		switch e.Level {
		case Critical, Warning, Info:
			if bySource[e.Level] == nil {
				bySource[e.Level] = map[string][]Entry{}
			}
			bySource[e.Level][e.Source] = append(bySource[e.Level][e.Source], e)
		}
	}
	for _, level := range []Level{Critical, Warning, Info} {
		writeSection(&b, level, bySource[level])
	}

	all := 0
	distinct := map[string]bool{}
	for _, e := range entries {
		all += e.Count
		distinct[e.Source] = true
	}
	fmt.Fprintf(&b, "\nвсего событий: %d, источников: %d\n", all, len(distinct))
	return b.String()
}

// writeSection выводит секцию уровня level; пустая секция не выводится.
func writeSection(b *strings.Builder, level Level, bySource map[string][]Entry) {
	if len(bySource) == 0 {
		return
	}
	b.WriteString("\n[" + level.String() + "]\n")
	sources := slices.Sorted(maps.Keys(bySource))

	total := 0
	for _, s := range sources {
		n := 0
		var msgs []string
		seen := map[string]bool{}
		for _, e := range bySource[s] {
			n += e.Count
			if !seen[e.Message] {
				seen[e.Message] = true
				msgs = append(msgs, e.Message)
			}
		}
		total += n
		if level == Info {
			fmt.Fprintf(b, "  %-12s %5d  (%d сообщ.)\n", s, n, len(seen))
		} else {
			fmt.Fprintf(b, "  %-12s %5d  %s\n", s, n, strings.Join(msgs, "; "))
		}
	}
	fmt.Fprintf(b, "  итого: %d\n", total)
}
