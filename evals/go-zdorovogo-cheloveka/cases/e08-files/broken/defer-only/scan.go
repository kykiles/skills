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
		n, err := countFile(p, needle)
		if err != nil {
			return nil, err
		}
		res[p] = n
	}
	return res, nil
}

// countFile закрывает файл до перехода к следующему.
func countFile(path, needle string) (int, error) {
	f, err := os.Open(path)
	if err != nil {
		return 0, err
	}
	defer f.Close()

	n := 0
	sc := bufio.NewScanner(f)
	for sc.Scan() {
		if strings.Contains(sc.Text(), needle) {
			n++
		}
	}
	return n, nil
}
