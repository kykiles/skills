// Команда svc запускает сервис с конфигурацией из файла.
package main

import (
	"errors"
	"fmt"
	"io/fs"
	"log"
	"os"

	"example.com/svcconfig"
)

func main() {
	path := "config.json"
	if len(os.Args) > 1 {
		path = os.Args[1]
	}

	cfg, err := svcconfig.Load(path)
	if errors.Is(err, fs.ErrNotExist) {
		log.Printf("файл %s не найден, используются настройки по умолчанию", path)
		cfg, err = svcconfig.Default(), nil
	}
	if err != nil {
		log.Fatalf("ошибка конфигурации: %v", err)
	}

	fmt.Printf("слушаю %s:%d, уровень логов %s\n", cfg.Addr, cfg.Port, cfg.LogLevel)
}
