package google

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"

	"github.com/Asutorufa/hujiang_dictionary/httpclient"
)

type Result struct {
	Target []string `json:"target"`
	Source []string `json:"source"`
}

func Translate(text, srcLang, tarLang string) (*Result, error) {
	if tarLang == "" {
		return nil, fmt.Errorf("target language is empty")
	}
	if srcLang == "" {
		srcLang = "auto"
	}

	q := url.Values{}
	q.Add("client", "gtx")
	q.Add("dt", "t")
	q.Add("sl", srcLang)
	q.Add("tl", tarLang)
	q.Add("q", text)

	req, err := http.NewRequest(http.MethodGet,
		"https://translate.googleapis.com/translate_a/single", nil)
	if err != nil {
		return nil, err
	}
	req.URL.RawQuery = q.Encode()

	resp, err := httpclient.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, errors.New(resp.Status)
	}

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("reading response body failed: %w", err)
	}

	var result []interface{}
	err = json.Unmarshal(body, &result)
	if err != nil {
		return nil, fmt.Errorf("unmarshal body failed: %w", err)
	}

	if len(result) <= 0 {
		return nil, errors.New("no translated data in response")
	}

	var resultText, sourceText []string
	for _, slice := range result[0].([]interface{}) {
		z, ok := slice.([]interface{})
		if !ok || len(z) < 2 {
			continue
		}
		resultText = append(resultText, fmt.Sprint(z[0]))
		sourceText = append(sourceText, fmt.Sprint(z[1]))
	}
	return &Result{resultText, sourceText}, nil
}
