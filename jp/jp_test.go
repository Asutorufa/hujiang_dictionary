package jp

import (
	"bytes"
	_ "embed"
	"fmt"
	"testing"
)

func TestGetJson(t *testing.T) {
	t.Log(GetJson("kodomo"))
}

func TestShow(t *testing.T) {
	t.Log(FormatString("魚"))
	// t.Log(FormatString("kodomo"))
}

//go:embed test_data.txt
var x []byte

func TestGetWord(t *testing.T) {
	w := getWords(bytes.NewReader(x))
	t.Log(convertToString(w))
}

func TestMarkdown(t *testing.T) {
	fmt.Println(FormatMarkdown("魚"))
}
