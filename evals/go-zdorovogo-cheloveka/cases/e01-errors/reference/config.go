// Package svcconfig загружает конфигурацию сервиса из JSON-файла.
package svcconfig

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"
)

// Config — настройки сервиса.
type Config struct {
	Addr     string `json:"addr"`
	Port     int    `json:"port"`
	LogLevel string `json:"log_level"`
}

// Default возвращает настройки по умолчанию.
func Default() *Config {
	cfg := &Config{Port: 8080}
	cfg.applyDefaults()
	return cfg
}

// ValidationError сообщает, какое поле конфигурации некорректно.
type ValidationError struct {
	Field  string
	Reason string
}

func (e *ValidationError) Error() string {
	return fmt.Sprintf("поле %s: %s", e.Field, e.Reason)
}

// Load читает и проверяет конфигурацию из файла path.
// При ошибке возвращает nil и ошибку.
func Load(path string) (*Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("чтение конфигурации %s: %w", path, err)
	}
	var cfg Config
	if err := json.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("разбор конфигурации %s: %w", path, err)
	}
	cfg.applyDefaults()
	if err := validate(&cfg); err != nil {
		return nil, err
	}
	return &cfg, nil
}

func (c *Config) applyDefaults() {
	if c.Addr == "" {
		c.Addr = "127.0.0.1"
	}
	if c.LogLevel == "" {
		c.LogLevel = "info"
	}
}

func validate(c *Config) *ValidationError {
	if c.Port < 1 || c.Port > 65535 {
		return &ValidationError{Field: "port", Reason: "должен быть от 1 до 65535"}
	}
	switch c.LogLevel {
	case "debug", "info", "warn", "error":
	default:
		return &ValidationError{Field: "log_level", Reason: "неизвестный уровень " + strconv.Quote(c.LogLevel)}
	}
	return nil
}
