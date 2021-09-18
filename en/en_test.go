package en

import (
	"testing"
)

func TestGet(t *testing.T) {
	for _, x := range Get("good") {
		t.Log(x.Word)
		t.Log(x.AudioEnUrl)
		t.Log(x.AudioUsUrl)
		t.Log(x.Katakana)
		t.Log(x.Roma)
		for _, x := range x.Phrase {
			t.Log(x)
		}
		t.Log(x.Synonym)
		t.Log(x.Antonym)
		t.Log(x.Inflections)
		t.Log(x.Simple)
	}
}

func TestGetJson(t *testing.T) {
	x, _ := GetJson("good")
	t.Log(x)
}

func TestShow(t *testing.T) {
	t.Log(FormatString("good"))
	t.Log(FormatString("show"))
}
