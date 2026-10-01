// Package cmdargs собирает аргументы командной строки для внешних утилит.
package cmdargs

// Build возвращает base, за которыми следуют extra.
// base не изменяется.
func Build(base []string, extra ...string) []string {
	return append(base, extra...)
}

// WithoutEmpty возвращает args без пустых строк, сохраняя порядок.
// args не изменяется.
func WithoutEmpty(args []string) []string {
	out := args[:0]
	for _, a := range args {
		if a != "" {
			out = append(out, a)
		}
	}
	return out
}
