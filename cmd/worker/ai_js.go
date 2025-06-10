package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"math/rand/v2"
	"strings"
	"syscall/js"

	tgbotapi "github.com/OvyFlash/telegram-bot-api"
	"github.com/syumai/go-jsutil"
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

	t, err := jsutil.AwaitPromise(p)
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
	obj := jsutil.NewObject()
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

type Llama2_7bChatOptions struct {
	Prompt string
}

func (opts *Llama2_7bChatOptions) toJS() js.Value {
	if opts == nil {
		return js.Undefined()
	}
	system := jsutil.NewObject()
	system.Set("role", "system")
	system.Set("content", "You are the professional translator. You need translate input text by user's instruction. Don't print with markdown format.")

	obj := jsutil.NewObject()
	obj.Set("role", "user")
	obj.Set("content", opts.Prompt)

	x := jsutil.NewObject()
	x.Set("messages", js.ValueOf([]any{system, obj}))
	// x.Set("prompt", opts.Prompt)
	x.Set("max_tokens", 1024)
	x.Set("seed", rand.IntN(999999))
	x.Set("stream", true)
	x.Set("temperature", 2.5)
	x.Set("top_k", 10)

	return x
}

// @cf/google/gemma-3-12b-it
func (a *AI) Gemma3_12b(opt Llama2_7bChatOptions) (io.ReadCloser, error) {
	p := a.instance.Call("run", "@cf/google/gemma-3-12b-it", opt.toJS())

	t, err := jsutil.AwaitPromise(p)
	if err != nil {
		return nil, err
	}

	return jsutil.ConvertReadableStreamToReadCloser(t), nil
}

// @cf/meta/llama-4-scout-17b-16e-instruct
func (a *AI) LLama4Scout17b16eInstruct(opt Llama2_7bChatOptions) (io.ReadCloser, error) {
	p := a.instance.Call("run", "@cf/meta/llama-4-scout-17b-16e-instruct", opt.toJS())

	t, err := jsutil.AwaitPromise(p)
	if err != nil {
		return nil, err
	}

	return jsutil.ConvertReadableStreamToReadCloser(t), nil
}

type llamaStreamDecoder struct {
	r *bufio.Scanner
}

func NewLlamaStreamDecoder(r io.Reader) *llamaStreamDecoder {
	dec := &llamaStreamDecoder{bufio.NewScanner(r)}
	return dec
}

func (l *llamaStreamDecoder) Decode() (string, error) {
	for l.r.Scan() {
		text := l.r.Text()
		if text == "" {
			continue
		}

		sections := strings.SplitN(text, ":", 2)
		field, value := sections[0], ""
		if len(sections) == 2 {
			value = strings.TrimPrefix(sections[1], " ")
		}
		switch field {
		case "event":
		case "data":
			if value == "[DONE]" {
				return "", io.EOF
			}

			x := map[string]any{}
			err := json.Unmarshal([]byte(value), &x)
			if err != nil {
				log.Println("json unmarshal", "err", err, "value", value)
				continue
			}

			if x["response"] == nil {
				continue
			}

			return fmt.Sprint(x["response"]), nil
		case "id":
		case "retry":
		}
	}

	return "", io.EOF
}

func ReturnByEventSource(r io.ReadCloser, update *tgbotapi.Message, argument string) {
	br := NewLlamaStreamDecoder(r)

	text := strings.Builder{}
	last, msgId, count := 0, 0, 0
	for {
		e, err := br.Decode()
		if err != nil {
			if err != io.EOF {
				log.Println("decode error", err)
			}
			break
		}

		if e == "" {
			continue
		}

		text.WriteString(e)

		if count >= 12 || text.Len()-last <= 300 {
			continue
		}

		msg, err := SendText(argument, update, msgId, text.String())
		if err != nil {
			log.Println("send text", "err", err)
			return
		}

		count++
		last = text.Len()

		if msgId == 0 {
			msgId = msg.MessageID
		}
	}

	SendText(argument, update, msgId, text.String())
}
