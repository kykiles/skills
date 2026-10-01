// Package ttlcache — небольшой кэш строк с временем жизни записей.
package ttlcache

import (
	"sync"
	"time"
)

type entry struct {
	value   string
	expires time.Time
}

// Cache безопасен для одновременного использования из нескольких горутин.
// Нулевое значение не готово к работе; используйте New.
type Cache struct {
	mu    sync.Mutex
	ttl   time.Duration
	items map[string]entry
}

// New создаёт кэш, в котором записи живут ttl.
func New(ttl time.Duration) *Cache {
	return &Cache{ttl: ttl, items: make(map[string]entry)}
}

// Set сохраняет value под ключом key.
func (c *Cache) Set(key, value string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.items[key] = entry{value: value, expires: time.Now().Add(c.ttl)}
}

// Get возвращает значение, если оно есть и не устарело.
func (c *Cache) Get(key string) (string, bool) {
	e, ok := c.items[key]
	if !ok || time.Now().After(e.expires) {
		return "", false
	}
	return e.value, true
}

// Len возвращает число записей, включая устаревшие, но ещё не удалённые.
func (c Cache) Len() int {
	c.mu.Lock()
	defer c.mu.Unlock()
	return len(c.items)
}

// Snapshot возвращает копию всех неустаревших записей.
func (c *Cache) Snapshot() map[string]string {
	c.mu.Lock()
	defer c.mu.Unlock()
	now := time.Now()
	out := make(map[string]string, len(c.items))
	for k, e := range c.items {
		if !now.After(e.expires) {
			out[k] = e.value
		}
	}
	return out
}

// StartCleanup каждые interval удаляет устаревшие записи, пока не вызвана stop.
func (c *Cache) StartCleanup(interval time.Duration) (stop func()) {
	ticker := time.NewTicker(interval)
	done := make(chan struct{})
	go func() {
		defer ticker.Stop()
		for {
			select {
			case <-ticker.C:
				c.mu.Lock()
				now := time.Now()
				for k, e := range c.items {
					if now.After(e.expires) {
						delete(c.items, k)
					}
				}
				c.mu.Unlock()
			case <-done:
				return
			}
		}
	}()
	return func() { close(done) }
}
