package report

//go:generate stringer -type=Level

// Level — уровень важности события.
type Level int

const (
	Info Level = iota
	Warning
	Critical
)
