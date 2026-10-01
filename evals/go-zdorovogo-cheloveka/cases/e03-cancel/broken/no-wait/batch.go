// Package batch выполняет обработку элементов параллельно.
package batch

import (
	"context"
	"sync"
)

// Result — результат обработки одного элемента.
type Result struct {
	Item  string
	Value int
}

// ProcessAll вызывает fn для каждого элемента items, используя не более workers
// параллельных вызовов. Результаты возвращаются в порядке items.
//
// При первой ошибке fn или при отмене ctx ProcessAll прекращает выдавать новые
// элементы и возвращает эту ошибку (или ctx.Err()). ProcessAll не возвращает
// управление, пока не завершатся все уже начатые вызовы fn.
func ProcessAll(ctx context.Context, items []string, workers int, fn func(context.Context, string) (int, error)) ([]Result, error) {
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	var (
		wg       sync.WaitGroup
		mu       sync.Mutex
		firstErr error
	)
	results := make([]Result, len(items))
	jobs := make(chan int)

	for w := 0; w < workers; w++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for i := range jobs {
				v, err := fn(ctx, items[i])
				if err != nil {
					mu.Lock()
					if firstErr == nil {
						firstErr = err
						cancel()
					}
					mu.Unlock()
					continue
				}
				results[i] = Result{Item: items[i], Value: v}
			}
		}()
	}

feed:
	for i := range items {
		select {
		case <-ctx.Done():
			break feed
		default:
		}
		select {
		case jobs <- i:
		case <-ctx.Done():
			break feed
		}
	}
	close(jobs)


	if firstErr != nil {
		return nil, firstErr
	}
	if err := ctx.Err(); err != nil {
		return nil, err
	}
	return results, nil
}
