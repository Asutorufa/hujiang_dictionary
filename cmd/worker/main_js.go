package main

import (
	"encoding/json"
	"fmt"
	"log"
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
	tgbotapi "github.com/OvyFlash/telegram-bot-api"
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
	workers.Serve(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer func() {
			if err := recover(); err != nil {
				log.Println("panic", "err", err)
				http.Error(w, fmt.Sprintf("%v", err), http.StatusInternalServerError)
			}
		}()
		log.Println("new request", "method", r.Method, "path", r.URL.Path)

		switch r.URL.Path {
		case "/tgbot":
			bot(w, r)
		case "/tgbot/register":
			register(w, r)
		default:
			defaultHandler(w, r)
		}
	})) // use http.DefaultServeMux
}

type Args struct {
	Text string
}

func translate(cmd string, args Args) []string {
	var resp []string
	argument := args.Text
	switch cmd {
	case "en":
		resp = en.FormatMarkdown(argument)
	case "jpcn":
		resp = jp.FormatMarkdown(argument)
	case "cnjp":
		resp = []string{jp.FormatCNString(argument)}
	case "ktbk":
		resp = []string{kotobakku.FormatString(argument)}
	case "ko":
		resp = []string{kr.FormatString(argument)}
	case "weblio":
		resp = []string{weblio.FormatString(argument)}
	default:
		if strings.HasPrefix(cmd, "cfai") {
			cmd = cmd[4:]
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
				resp = []string{err.Error()}
			} else {
				resp = []string{str}
			}

			return resp
		}

		if strings.HasPrefix(cmd, "gg") {
			cmd = cmd[2:]
			str, err := google.Translate(argument, "", cmd)
			if err != nil {
				resp = []string{err.Error()}
			} else {
				resp = str.Target
			}

			return resp
		}
	}

	return resp
}

func bot(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		http.NotFound(w, r)
		return
	}

	var update tgbotapi.Update
	err := json.NewDecoder(r.Body).Decode(&update)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	authorized := false
	for _, id := range strings.FieldsFunc(cloudflare.Getenv("telegram_ids"), func(r rune) bool { return r == ',' }) {
		i, err := strconv.ParseInt(id, 10, 64)
		if err != nil {
			log.Println("parse id", "id", id, "err", err)
			continue
		}

		if update.Message.From.ID == i || update.Message.Chat.ID == i {
			authorized = true
			break
		}
	}

	if update.Message == nil || !authorized {
		return
	}

	log.Println("new message",
		"text", replaceLineBreak(update.Message.Text),
		"user id", update.Message.From.ID, "user", update.Message.From.UserName,
		"group id", update.Message.Chat.ID, "group", update.Message.Chat.Title,
	)

	if update.Message.ReplyToMessage != nil {
		log.Println("reply to message",
			"text", replaceLineBreak(update.Message.ReplyToMessage.Text),
			"user id", update.Message.ReplyToMessage.From.ID, "user", update.Message.ReplyToMessage.From.UserName,
		)

		if update.Message.ReplyToMessage.Quote != nil {
			log.Println("reply to quote message",
				"text", replaceLineBreak(update.Message.ReplyToMessage.Quote.Text),
				"user id", update.Message.ReplyToMessage.From.ID, "user", update.Message.ReplyToMessage.From.UserName,
			)
		}
	}

	if update.Message.Quote != nil {
		log.Println("quote message",
			"text", replaceLineBreak(update.Message.Quote.Text),
			"user id", update.Message.From.ID, "user", update.Message.From.UserName,
		)
	}

	argument := update.Message.CommandArguments()

	if argument == "" {
		if update.Message.Quote != nil {
			log.Println("use quote message", "text", replaceLineBreak(update.Message.Quote.Text))
			argument = update.Message.Quote.Text
		} else if update.Message.ReplyToMessage != nil {
			if update.Message.ReplyToMessage.Quote != nil {
				log.Println("use reply quote message", "text", replaceLineBreak(update.Message.ReplyToMessage.Quote.Text))
				argument = update.Message.ReplyToMessage.Quote.Text
			} else {
				log.Println("use reply message", "text", replaceLineBreak(update.Message.ReplyToMessage.Text))
				argument = update.Message.ReplyToMessage.Text
			}
		} else {
			return
		}
	}

	resp := translate(update.Message.Command(), Args{Text: argument})

	for _, r := range resp {
		msg := tgbotapi.NewMessage(update.Message.Chat.ID, r)
		msg.ReplyParameters.MessageID = update.Message.MessageID
		_, err = Bot.Request(msg)
		if err != nil {
			log.Println("send message", "err", err)
		}
	}
}

type writerWrapper struct {
	http.ResponseWriter
	isWritten bool
}

func (w *writerWrapper) Write([]byte) (int, error) {
	w.isWritten = true
	return 0, nil
}

func register(w http.ResponseWriter, r *http.Request) {
	wh, err := tgbotapi.NewWebhook(cloudflare.Getenv("worker_url") + "/tgbot")
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	_, err = Bot.Request(wh)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
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
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	json.NewEncoder(w).Encode(resp)
}

func defaultHandler(w http.ResponseWriter, req *http.Request) {
	t := req.URL.Query().Get("type")
	word := req.URL.Query().Get("word")

	if t == "" || word == "" {
		w.WriteHeader(http.StatusBadRequest)
		w.Write([]byte("bad request"))
		return
	}

	log.Println("translate", "type", t, "word", word)

	w.Header().Set("Content-Type", "text/plain; charset=utf-8")

	resp := translate(t, Args{Text: word})
	if len(resp) == 0 {
		w.WriteHeader(http.StatusNotFound)
		w.Write([]byte("not found"))
		return
	}

	for _, r := range resp {
		w.Write([]byte(r + "\n"))
	}
}

func replaceLineBreak(s string) string {
	return strings.ReplaceAll(s, "\n", "\\n")
}
