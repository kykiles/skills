// Package statusapi — клиент сервиса статусов задач.
package statusapi

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"net/url"
)

// Status — состояние задачи.
type Status struct {
	ID    string `json:"id"`
	State string `json:"state"`
}

// ErrNotFound возвращается, если задачи с таким ID нет (HTTP 404).
var ErrNotFound = errors.New("statusapi: задача не найдена")

// StatusError — ответ сервиса с неожиданным HTTP-статусом.
type StatusError struct {
	Code int
}

func (e *StatusError) Error() string {
	return fmt.Sprintf("statusapi: неожиданный HTTP-статус %d", e.Code)
}

// Client обращается к сервису статусов.
type Client struct {
	BaseURL string       // например, "http://localhost:8080"
	HTTP    *http.Client // nil означает http.DefaultClient
}

func (c *Client) httpClient() *http.Client {
	if c.HTTP != nil {
		return c.HTTP
	}
	return http.DefaultClient
}

// GetStatus запрашивает статус задачи id.
// Для ответа 404 возвращает ErrNotFound, для остальных статусов, кроме 200, —
// *StatusError. Учитывает отмену и дедлайн ctx.
func (c *Client) GetStatus(ctx context.Context, id string) (Status, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, c.BaseURL+"/v1/status/"+url.PathEscape(id), nil)
	if err != nil {
		return Status{}, err
	}
	resp, err := c.httpClient().Do(req)
	if err != nil {
		return Status{}, err
	}

	switch resp.StatusCode {
	case http.StatusOK:
	case http.StatusNotFound:
		return Status{}, ErrNotFound
	default:
		return Status{}, &StatusError{Code: resp.StatusCode}
	}

	var st Status
	if err := json.NewDecoder(resp.Body).Decode(&st); err != nil {
		return Status{}, fmt.Errorf("statusapi: разбор ответа: %w", err)
	}
	return st, nil
}
