// Package batch выполняет обработку элементов параллельно.
package batch

import "context"

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
	type job struct {
		index int
		item  string
	}
	type outcome struct {
		index int
		value int
		err   error
	}

	jobs := make(chan job)
	outcomes := make(chan outcome)

	for w := 0; w < workers; w++ {
		go func() {
			for j := range jobs {
				v, err := fn(ctx, j.item)
				outcomes <- outcome{j.index, v, err}
			}
		}()
	}

	go func() {
		for i, item := range items {
			jobs <- job{i, item}
		}
		close(jobs)
	}()

	results := make([]Result, len(items))
	for range items {
		select {
		case o := <-outcomes:
			if o.err != nil {
				return nil, o.err
			}
			results[o.index] = Result{Item: items[o.index], Value: o.value}
		case <-ctx.Done():
			return nil, ctx.Err()
		}
	}
	return results, nil
}
