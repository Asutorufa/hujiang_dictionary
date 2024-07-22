package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"strings"

	"github.com/Asutorufa/hujiang_dictionary/en"
	"github.com/Asutorufa/hujiang_dictionary/httpclient"
	"github.com/Asutorufa/hujiang_dictionary/jp"
	"github.com/Asutorufa/hujiang_dictionary/kotobakku"
	"github.com/Asutorufa/hujiang_dictionary/kr"
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
*/

func main() {
	httpclient.DefaultClient = httputil.DefaultClient

	bot()

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

		info, err := Bot.GetWebhookInfo()

		json.NewEncoder(w).Encode([]any{
			info, err,
		})

		Bot.Request(tgbotapi.NewSetMyCommands(
			tgbotapi.BotCommand{Command: "en", Description: "en"},
			tgbotapi.BotCommand{Command: "jpcn", Description: "jp -> cn"},
			tgbotapi.BotCommand{Command: "cnjp", Description: "cn -> jp"},
			tgbotapi.BotCommand{Command: "ktbk", Description: "コトバック"},
			tgbotapi.BotCommand{Command: "kr", Description: "kr"},
		))
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

		switch t {
		case "jp":
			w.Write([]byte(jp.FormatString(word)))
		case "cnjp":
			w.Write([]byte(jp.FormatCNString(word)))
		case "kr":
			w.Write([]byte(kr.FormatString(word)))
		case "ktbk":
			w.Write([]byte(kotobakku.FormatString(word)))
		case "en":
			w.Write([]byte(en.FormatString(word)))
		default:
			w.WriteHeader(http.StatusInternalServerError)
			w.Write([]byte("unsupported type"))
		}
	})
	workers.Serve(nil) // use http.DefaultServeMux
}

func bot() {
	idMap := make(map[int64]bool)
	for _, id := range strings.FieldsFunc(cloudflare.Getenv("telegram_ids"), func(r rune) bool { return r == ',' }) {
		i, err := strconv.ParseInt(id, 10, 64)
		if err != nil {
			fmt.Println(err)
			continue
		}

		idMap[i] = true
	}

	http.HandleFunc("/tgbot", func(w http.ResponseWriter, r *http.Request) {
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

		var resp string
		switch update.Message.Command() {
		case "en":
			resp = en.FormatString(argument)
		case "jpcn":
			resp = jp.FormatString(argument)
		case "cnjp":
			resp = jp.FormatCNString(argument)
		case "ktbk":
			resp = kotobakku.FormatString(argument)
		case "kr":
			resp = kr.FormatString(argument)
		default:
			return
		}

		if resp == "" {
			return
		}

		msg := tgbotapi.NewMessage(update.Message.Chat.ID, resp)
		msg.ReplyToMessageID = update.Message.MessageID

		Bot.Send(msg)
	})
}
