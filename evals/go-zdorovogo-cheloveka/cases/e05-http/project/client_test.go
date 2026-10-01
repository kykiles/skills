package statusapi

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestGetStatus(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte(`{"id":"42","state":"done"}`))
	}))
	defer srv.Close()

	c := &Client{BaseURL: srv.URL}
	st, err := c.GetStatus(context.Background(), "42")
	if err != nil {
		t.Fatal(err)
	}
	if st != (Status{ID: "42", State: "done"}) {
		t.Fatalf("GetStatus = %+v", st)
	}
}
