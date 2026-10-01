package statusapi_test

import (
	"context"
	"errors"
	"io"
	"net/http"
	"net/http/httptest"
	"sync/atomic"
	"testing"
	"time"

	"example.com/statusapi"
)

type graderBody struct {
	io.ReadCloser
	closed *atomic.Int64
	once   atomic.Bool
}

func (b *graderBody) Close() error {
	if b.once.CompareAndSwap(false, true) {
		b.closed.Add(1)
	}
	return b.ReadCloser.Close()
}

type graderTransport struct {
	opened, closed atomic.Int64
}

func (tr *graderTransport) RoundTrip(r *http.Request) (*http.Response, error) {
	resp, err := http.DefaultTransport.RoundTrip(r)
	if err != nil {
		return nil, err
	}
	tr.opened.Add(1)
	resp.Body = &graderBody{ReadCloser: resp.Body, closed: &tr.closed}
	return resp, nil
}

func graderServer(t *testing.T) *httptest.Server {
	t.Helper()
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.URL.EscapedPath() {
		case "/v1/status/ok":
			w.Header().Set("Content-Type", "application/json")
			io.WriteString(w, `{"id":"ok","state":"done"}`)
		case "/v1/status/missing":
			w.Header().Set("Content-Type", "application/json")
			w.WriteHeader(http.StatusNotFound)
			io.WriteString(w, `{"error":"not found"}`)
		case "/v1/status/boom":
			http.Error(w, "internal", http.StatusInternalServerError)
		case "/v1/status/slow":
			select {
			case <-r.Context().Done():
			case <-time.After(5 * time.Second):
			}
			io.WriteString(w, `{"id":"slow","state":"late"}`)
		case "/v1/status/a%20b%2Fc":
			io.WriteString(w, `{"id":"a b/c","state":"escaped"}`)
		default:
			http.Error(w, "unexpected path: "+r.URL.EscapedPath(), http.StatusBadRequest)
		}
	}))
	t.Cleanup(srv.Close)
	return srv
}

func graderClient(t *testing.T) (*statusapi.Client, *graderTransport) {
	srv := graderServer(t)
	tr := &graderTransport{}
	return &statusapi.Client{BaseURL: srv.URL, HTTP: &http.Client{Transport: tr}}, tr
}

func graderClosed(t *testing.T, tr *graderTransport) {
	t.Helper()
	if o, c := tr.opened.Load(), tr.closed.Load(); o != c {
		t.Fatalf("response bodies: opened %d, closed %d", o, c)
	}
}

func TestGraderOK(t *testing.T) {
	c, tr := graderClient(t)
	st, err := c.GetStatus(context.Background(), "ok")
	if err != nil || st != (statusapi.Status{ID: "ok", State: "done"}) {
		t.Fatalf("GetStatus(ok) = %+v, %v", st, err)
	}
	graderClosed(t, tr)
}

func TestGraderNotFound(t *testing.T) {
	c, tr := graderClient(t)
	_, err := c.GetStatus(context.Background(), "missing")
	if !errors.Is(err, statusapi.ErrNotFound) {
		t.Fatalf("err = %v; want ErrNotFound", err)
	}
	graderClosed(t, tr)
}

func TestGraderServerError(t *testing.T) {
	c, tr := graderClient(t)
	_, err := c.GetStatus(context.Background(), "boom")
	var se *statusapi.StatusError
	if !errors.As(err, &se) || se.Code != http.StatusInternalServerError {
		t.Fatalf("err = %v; want *StatusError{500}", err)
	}
	graderClosed(t, tr)
}

func TestGraderDeadline(t *testing.T) {
	c, tr := graderClient(t)
	ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
	defer cancel()
	start := time.Now()
	_, err := c.GetStatus(ctx, "slow")
	if !errors.Is(err, context.DeadlineExceeded) {
		t.Fatalf("err = %v; want context.DeadlineExceeded", err)
	}
	if d := time.Since(start); d > 2*time.Second {
		t.Fatalf("GetStatus ignored deadline: took %v", d)
	}
	graderClosed(t, tr)
}

func TestGraderCanceledBeforeCall(t *testing.T) {
	c, _ := graderClient(t)
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	if _, err := c.GetStatus(ctx, "ok"); !errors.Is(err, context.Canceled) {
		t.Fatalf("err = %v; want context.Canceled", err)
	}
}

func TestGraderEscapingKept(t *testing.T) {
	c, tr := graderClient(t)
	st, err := c.GetStatus(context.Background(), "a b/c")
	if err != nil || st.State != "escaped" {
		t.Fatalf("GetStatus(a b/c) = %+v, %v", st, err)
	}
	graderClosed(t, tr)
}

func TestGraderNilHTTPClient(t *testing.T) {
	srv := graderServer(t)
	c := &statusapi.Client{BaseURL: srv.URL}
	if st, err := c.GetStatus(context.Background(), "ok"); err != nil || st.ID != "ok" {
		t.Fatalf("GetStatus with nil HTTP = %+v, %v", st, err)
	}
}
