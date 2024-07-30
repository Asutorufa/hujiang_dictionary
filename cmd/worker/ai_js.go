package main

import (
	"syscall/js"
	_ "unsafe"

	_ "github.com/syumai/workers"
	"github.com/syumai/workers/cloudflare"
)

type AI struct {
	instance js.Value
}

func NewAI() *AI {
	return &AI{
		instance: cloudflare.GetBinding("AI"),
	}
}

func (a *AI) Translate(opts TranslateOptions) (string, error) {
	p := a.instance.Call("run", "@cf/meta/m2m100-1.2b", opts.toJS())

	t, err := AwaitPromise(p)
	if err != nil {
		return "", err
	}

	return t.Get("translated_text").String(), nil
}

/*
"@cf/meta/m2m100-1.2b",

	{
	  text: "I'll have an order of the moule frites",
	  source_lang: "english", // defaults to english
	  target_lang: "french",
	}


	​​Response

	{
	  "translated_text": "Je vais commander des moules frites"
	}
*/
type TranslateOptions struct {
	Text       string
	SourceLang string
	TargetLang string
}

func (opts *TranslateOptions) toJS() js.Value {
	if opts == nil {
		return js.Undefined()
	}
	obj := NewObject()
	if opts.Text != "" {
		obj.Set("text", opts.Text)
	}
	if opts.SourceLang != "" {
		obj.Set("source_lang", opts.SourceLang)
	}
	if opts.TargetLang != "" {
		obj.Set("target_lang", opts.TargetLang)
	}
	return obj
}

//go:linkname NewObject github.com/syumai/workers/internal/jsutil.NewObject
func NewObject() js.Value

//go:linkname AwaitPromise github.com/syumai/workers/internal/jsutil.AwaitPromise
func AwaitPromise(promiseVal js.Value) (js.Value, error)
