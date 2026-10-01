// Package cmdargs собирает аргументы командной строки для внешних утилит.
package cmdargs

// Build возвращает base, за которыми следуют extra.
// base не изменяется.
func Build(base []string, extra ...string) []string {
	out := make([]string, 0, len(base)+len(extra))
	out = append(out, base...)
	return append(out, extra...)
}

// WithoutEmpty возвращает args без пустых строк, сохраняя порядок.
// args не изменяется.
func WithoutEmpty(args []string) []string {
	out := make([]string, 0, len(args))
	for _, a := range args {
		if a != "" {
			out = append(out, a)
		}
	}
	return out
}
