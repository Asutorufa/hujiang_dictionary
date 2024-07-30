package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"strings"

	"github.com/Asutorufa/hujiang_dictionary/en"
	"github.com/Asutorufa/hujiang_dictionary/google"
	"github.com/Asutorufa/hujiang_dictionary/httpclient"
	"github.com/Asutorufa/hujiang_dictionary/jp"
	"github.com/Asutorufa/hujiang_dictionary/kotobakku"
	"github.com/Asutorufa/hujiang_dictionary/kr"
	"github.com/Asutorufa/hujiang_dictionary/weblio"
	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api/v5"
	"github.com/syumai/tinyutil/httputil"
	"github.com/syumai/workers"
	"github.com/syumai/workers/cloudflare"
)

var Bot = &tgbotapi.BotAPI{
	Token:  cloudflare.Getenv("telegram_token"),
	Client: httputil.DefaultClient,
	Buffer: 100,
}

func init() {
	Bot.SetAPIEndpoint(tgbotapi.APIEndpoint)
	// Bot.Debug = true
}

/*
wrangler.toml

name = "hj-dict"
main = "./build/worker.mjs"
compatibility_date = "2024-04-15"

[build]
command = "make build"

[vars]
telegram_token = "****:*****"
worker_url = "https://*****.workers.dev"
telegram_ids = "40xxxxxx,42xxxxx"

[ai]
binding = "AI"
*/

func main() {
	httpclient.DefaultClient = httputil.DefaultClient

	http.HandleFunc("/tgbot", bot())

	http.HandleFunc("/tgbot/register", func(w http.ResponseWriter, r *http.Request) {
		wh, err := tgbotapi.NewWebhook(cloudflare.Getenv("worker_url") + "/tgbot")
		if err != nil {
			json.NewEncoder(w).Encode([]any{
				err.Error(),
			})
			return
		}

		_, err = Bot.Request(wh)
		if err != nil {
			json.NewEncoder(w).Encode([]any{
				err.Error(),
			})
			return
		}

		resp, err := Bot.Request(tgbotapi.NewSetMyCommands(
			tgbotapi.BotCommand{Command: "en", Description: "en"},
			tgbotapi.BotCommand{Command: "jpcn", Description: "jp -> cn"},
			tgbotapi.BotCommand{Command: "cnjp", Description: "cn -> jp"},
			tgbotapi.BotCommand{Command: "ktbk", Description: "コトバック"},
			tgbotapi.BotCommand{Command: "weblio", Description: "weblio辞書"},
			tgbotapi.BotCommand{Command: "ko", Description: "korean"},
			tgbotapi.BotCommand{Command: "cfaija", Description: "cloudflare worker ai -> japanese"},
			tgbotapi.BotCommand{Command: "cfaicn", Description: "cloudflare worker ai -> chinese"},
			tgbotapi.BotCommand{Command: "cfaitar_lang", Description: "cloudflare worker ai -> [tar_lang]"},
			tgbotapi.BotCommand{Command: "cfaisrc_lang2tar_lang", Description: "cloudflare worker ai src_lang -> tar_lang"},
			tgbotapi.BotCommand{Command: "ggtar_lang", Description: "google translate to tar_lang"},
		))

		if err != nil {
			json.NewEncoder(w).Encode([]any{
				err.Error(),
			})
			return
		}

		json.NewEncoder(w).Encode(resp)
	})

	http.HandleFunc("/", func(w http.ResponseWriter, req *http.Request) {
		t := req.URL.Query().Get("type")
		word := req.URL.Query().Get("word")

		if t == "" || word == "" {
			w.WriteHeader(http.StatusBadRequest)
			w.Write([]byte("bad request"))
			return
		}

		fmt.Println(t, word)

		w.Header().Set("Content-Type", "text/plain; charset=utf-8")

		resp := translate(t, Args{Text: word})
		if resp == "" {
			w.WriteHeader(http.StatusNotFound)
			w.Write([]byte("not found"))
			return
		}

		w.Write([]byte(resp))
	})
	workers.Serve(nil) // use http.DefaultServeMux
}

type Args struct {
	Text string
}

func translate(cmd string, args Args) string {
	var resp string
	argument := args.Text

	if strings.HasPrefix(cmd, "cfai") {
		cmd = strings.TrimPrefix(cmd, "cfai")
		var src string
		target := cmd
		if i := strings.IndexByte(cmd, '2'); i != -1 {
			src = cmd[:i]
			target = cmd[i+1:]
		}

		switch target {
		case "en":
			target = "english"
		case "jp":
			target = "japanese"
		case "cn":
			target = "chinese"
		}

		str, err := NewAI().Translate(TranslateOptions{
			Text:       argument,
			SourceLang: src,
			TargetLang: target,
		})
		if err != nil {
			resp = err.Error()
		} else {
			resp = str
		}

		return resp
	}

	if strings.HasPrefix(cmd, "gg") {
		cmd = strings.TrimPrefix(cmd, "gg")
		str, err := google.Translate(argument, "", cmd)
		if err != nil {
			resp = err.Error()
		} else {
			resp = strings.Join(str.Target, "\n")
		}

		return resp
	}

	switch cmd {
	case "en":
		resp = en.FormatString(argument)
	case "jpcn":
		resp = jp.FormatString(argument)
	case "cnjp":
		resp = jp.FormatCNString(argument)
	case "ktbk":
		resp = kotobakku.FormatString(argument)
	case "ko":
		resp = kr.FormatString(argument)
	case "weblio":
		resp = weblio.FormatString(argument)
	}

	return resp
}

func bot() func(w http.ResponseWriter, r *http.Request) {
	idMap := make(map[int64]bool)
	for _, id := range strings.FieldsFunc(cloudflare.Getenv("telegram_ids"), func(r rune) bool { return r == ',' }) {
		i, err := strconv.ParseInt(id, 10, 64)
		if err != nil {
			fmt.Println(err)
			continue
		}

		idMap[i] = true
	}

	return func(w http.ResponseWriter, r *http.Request) {
		update, err := Bot.HandleUpdate(r)
		if err != nil {
			json.NewEncoder(w).Encode([]any{
				err.Error(),
			})
			return
		}

		if update.Message == nil || (idMap != nil && !idMap[update.Message.From.ID]) {
			return
		}

		// If we got a message
		fmt.Printf("[%s] %s", update.Message.From.UserName, update.Message.Text)

		argument := update.Message.CommandArguments()

		if argument == "" {
			return
		}

		resp := translate(update.Message.Command(), Args{Text: argument})
		if resp == "" {
			return
		}

		msg := tgbotapi.NewMessage(update.Message.Chat.ID, resp)
		msg.ReplyToMessageID = update.Message.MessageID

		Bot.Send(msg)
	}
}
