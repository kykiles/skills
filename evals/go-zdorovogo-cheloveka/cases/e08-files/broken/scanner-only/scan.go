// Package logscan ищет строки в лог-файлах.
package logscan

import (
	"bufio"
	"os"
	"strings"
)

// CountMatches возвращает для каждого файла из paths число строк, содержащих
// needle. Если какой-то файл не удалось прочитать, возвращает nil и ошибку.
func CountMatches(paths []string, needle string) (map[string]int, error) {
	res := make(map[string]int, len(paths))
	for _, p := range paths {
		f, err := os.Open(p)
		if err != nil {
			return nil, err
		}
		defer f.Close()

		n := 0
		sc := bufio.NewScanner(f)
		sc.Buffer(make([]byte, 0, 64<<10), 1<<20)
		for sc.Scan() {
			if strings.Contains(sc.Text(), needle) {
				n++
			}
		}
		res[p] = n
	}
	return res, nil
}
