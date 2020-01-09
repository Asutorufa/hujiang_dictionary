package en

import "testing"

func TestGet(t *testing.T) {
	for _, x := range Get("good") {
		t.Log(x.Word)
		t.Log(x.AudioEnUrl)
		t.Log(x.AudioUsUrl)
		t.Log(x.Katakana)
		t.Log(x.Roma)
		t.Log(x.EnglishExplains)
		for _, x := range x.Phrase {
			t.Log(x)
		}
		t.Log(x.Synonym)
		t.Log(x.Antonym)
		t.Log(x.Inflections)
		t.Log(x.Simple)
		t.Log(x.Detail)
	}
}

func TestGetJson(t *testing.T) {
	x, _ := GetJson("good")
	t.Log(x)
}
