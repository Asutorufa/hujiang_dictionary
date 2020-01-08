package en

import "testing"

func TestGet(t *testing.T) {
	for _,x := range Get("good"){
		t.Log(x.Word)
		t.Log(x.AudioEnUrl)
		t.Log(x.AudioUsUrl)
		t.Log(x.Katakana)
		t.Log(x.Roma)
		t.Log(x.EnglishExplains)
		for _,x := range x.phrase{
			t.Log(x)
		}
		t.Log(x.synonym)
		t.Log(x.antonym)
		t.Log(x.inflections)
		t.Log(x.Simple)
		t.Log(x.Detail)
	}
}
