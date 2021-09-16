package jp

import (
	"bytes"
	_ "embed"
	"testing"
)

func TestGetJson(t *testing.T) {
	t.Log(GetJson("kodomo"))
}

func TestShow(t *testing.T) {
	t.Log(FormatString("さまざま"))
	t.Log(FormatString("kodomo"))
}

//go:embed test_data.html
var x []byte

func TestGetWord(t *testing.T) {
	w := getWords(bytes.NewReader(x))
	t.Log(convertToString(w))
}
