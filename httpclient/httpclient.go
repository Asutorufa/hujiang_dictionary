package httpclient

import "net/http"

type Client interface {
	Get(url string) (resp *http.Response, err error)
	Do(req *http.Request) (*http.Response, error)
}

var DefaultClient Client = &http.Client{}
