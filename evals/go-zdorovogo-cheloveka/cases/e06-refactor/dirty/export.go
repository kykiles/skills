package report

import (
	"strconv"
	"strings"
)

// FormatCSV — черновик экспорта в CSV (в работе, Маша).
func FormatCSV(entries []Entry) string {
	var b strings.Builder
	b.WriteString("source,level,message,count\n")
	for _,e := range entries {
		// TODO: экранирование запятых и кавычек
		b.WriteString(e.Source+","+e.Level.String()+","+e.Message+","+strconv.Itoa(e.Count)+"\n")
	}
	return b.String()
}
