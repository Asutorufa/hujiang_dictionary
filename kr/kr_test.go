package kr

import (
	"bytes"
	_ "embed"
	"testing"
)

//go:embed test_data2.txt
var testData []byte

func TestGetWord(t *testing.T) {
	w := getWords(bytes.NewReader(testData))
	t.Log(convertToString(w))
}
