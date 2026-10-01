package batch

import (
	"context"
	"testing"
)

func TestProcessAll(t *testing.T) {
	items := []string{"a", "bb", "ccc"}
	got, err := ProcessAll(context.Background(), items, 2, func(_ context.Context, s string) (int, error) {
		return len(s), nil
	})
	if err != nil {
		t.Fatal(err)
	}
	for i, r := range got {
		if r.Item != items[i] || r.Value != len(items[i]) {
			t.Fatalf("result %d = %+v", i, r)
		}
	}
}
