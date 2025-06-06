package main

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"time"

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
	"github.com/syumai/workers/cloudflare/cache"
	"github.com/syumai/workers/cloudflare/cron"
	_ "github.com/syumai/workers/cloudflare/d1"
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
	cron.ScheduleTaskNonBlock(func(ctx context.Context) error {
		ankiIdstr := cloudflare.Getenv("anki_telegram_id")
		if ankiIdstr == "" {
			log.Println("anki id is empty")
			return nil
		}

		ankiId, err := strconv.ParseInt(ankiIdstr, 10, 64)
		if err != nil {
			log.Println("parse anki id", "err", err)
			return nil
		}

		db, err := sql.Open("d1", "DB")
		if err != nil {
			return err
		}
		defer db.Close()

		result, err := db.Query("SELECT word, explain FROM words ORDER BY RANDOM() LIMIT 1")
		if err != nil {
			return err
		}

		result.Next()

		var word, explain string
		err = result.Scan(&word, &explain)
		if err != nil {
			return err
		}
		defer result.Close()

		msg := tgbotapi.NewMessage(ankiId, fmt.Sprintf(`<b>%s</b>
	<tg-spoiler><blockquote expandable>%s</blockquote></tg-spoiler>
	`, tgbotapi.EscapeText(tgbotapi.ModeHTML, word),
			tgbotapi.EscapeText(tgbotapi.ModeHTML, explain)))
		msg.ParseMode = tgbotapi.ModeHTML
		msg.LinkPreviewOptions.IsDisabled = true
		_, err = Bot.Send(msg)
		if err != nil {
			log.Println("send message failed", "err", err)
		}
		return err
	})

	httpclient.DefaultClient = httputil.DefaultClient
	workers.ServeNonBlock(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
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

	// send a ready signal to the runtime
	workers.Ready()

	// block until the handler or task is done
	select {
	case <-workers.Done():
	case <-cron.Done():
	}
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
		result := jp.FormatCNString(argument)
		if result != "" {
			resp = []string{result}
		}
	case "ktbk":
		result := kotobakku.FormatString(argument)
		if result != "" {
			resp = []string{result}
		}
	case "ko":
		result := kr.FormatString(argument)
		if result != "" {
			resp = []string{result}
		}
	case "weblio":
		result := weblio.FormatString(argument)
		if result != "" {
			resp = []string{result}
		}

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

	var umsg *tgbotapi.Message
	var edit bool
	var callback callbackQuery
	if update.Message != nil {
		umsg = update.Message
	} else if update.CallbackQuery != nil {
		umsg = update.CallbackQuery.Message
		i := strings.IndexByte(update.CallbackData(), ':')
		if i != -1 {
			callback = callbackQuery{
				Command: update.CallbackData()[:i],
				Data:    update.CallbackData()[i+1:],
			}
		} else {
			callback = callbackQuery{
				Command: update.CallbackData(),
			}
		}
	} else if update.EditedMessage != nil {
		umsg = update.EditedMessage
		edit = true
	} else {
		return
	}

	authorized := false
	for _, id := range strings.FieldsFunc(cloudflare.Getenv("telegram_ids"), func(r rune) bool { return r == ',' }) {
		i, err := strconv.ParseInt(id, 10, 64)
		if err != nil {
			log.Println("parse id", "id", id, "err", err)
			continue
		}

		if umsg.From.ID == i || (!callback.IsSave() && !callback.IsRemove() && umsg.Chat.ID == i) {
			authorized = true
			break
		}
	}

	if callback.Command != "" {
		if umsg == nil {
			return
		}

		switch callback.Command {
		case "delete":
			deleteMessage(update.CallbackQuery.Message, true)
			return
		case "save", "remove":
			var word string = callback.Data

			if word == "" {
				if umsg.ReplyToMessage == nil {
					return
				}

				word = umsg.ReplyToMessage.CommandArguments()
				if word == "" {
					log.Println("callback reply to message is empty")
					return
				}
			}

			db, err := sql.Open("d1", "DB")
			if err != nil {
				log.Println("open db failed", "err", err)
				return
			}
			defer db.Close()

			if callback.IsSave() {
				now := time.Now().Unix()
				/*
					INSERT INTO your_table (id, name, value)
					VALUES (1, 'Alice', 42)
					ON CONFLICT(id) DO UPDATE SET
					    name = excluded.name,
					    value = excluded.value;
				*/
				_, err = db.Exec("INSERT INTO words (word, explain, add_time, update_time) VALUES (?, ?, ?, ?) ON CONFLICT(word) DO UPDATE SET explain = ?, update_time = ?",
					word, update.CallbackQuery.Message.Text, now, now, update.CallbackQuery.Message.Text, now)
			} else {
				_, err = db.Exec("DELETE FROM words WHERE word = ?", word)
			}
			if err != nil {
				log.Println(callback.Command, "word failed", "word", word, "err", err)
			} else {
				log.Println(callback.Command, "word success", "word", word)
				editMessage(umsg, callback.Data, !callback.IsSave())
			}

			return
		}
	}

	if umsg == nil || !authorized {
		return
	}

	command := umsg.Command()

	switch command {
	case "delete":
		deleteMessage(umsg, false)
		return
	case "random":
		db, err := sql.Open("d1", "DB")
		if err != nil {
			log.Println("open db failed", "err", err)
			return
		}
		defer db.Close()

		result, err := db.Query("SELECT word, explain FROM words ORDER BY RANDOM() LIMIT 1")
		if err != nil {
			log.Println("get random word failed", "err", err)
			return
		}

		result.Next()

		var word, explain string
		err = result.Scan(&word, &explain)
		if err != nil {
			log.Println("scan word failed", "err", err)
			return
		}
		defer result.Close()

		msg := tgbotapi.NewMessage(umsg.Chat.ID, fmt.Sprintf(`<b>%s</b>
<tg-spoiler><blockquote expandable>%s</blockquote></tg-spoiler>
`, tgbotapi.EscapeText(tgbotapi.ModeHTML, word),
			tgbotapi.EscapeText(tgbotapi.ModeHTML, explain)))
		msg.ParseMode = tgbotapi.ModeHTML
		msg.LinkPreviewOptions.IsDisabled = true
		msg.ReplyParameters.MessageID = umsg.MessageID
		_, err = Bot.Send(msg)
		if err != nil {
			log.Println("send message failed", "err", err)
		}

		return
	}

	log.Println("new message",
		"text", replaceLineBreak(umsg.Text),
		"user id", umsg.From.ID, "user", umsg.From.UserName,
		"group id", umsg.Chat.ID, "group", umsg.Chat.Title,
	)

	if umsg.ReplyToMessage != nil {
		log.Println("reply to message",
			"text", replaceLineBreak(umsg.ReplyToMessage.Text),
			"user id", umsg.ReplyToMessage.From.ID, "user", umsg.ReplyToMessage.From.UserName,
		)

		if umsg.ReplyToMessage.Quote != nil {
			log.Println("reply to quote message",
				"text", replaceLineBreak(umsg.ReplyToMessage.Quote.Text),
				"user id", umsg.ReplyToMessage.From.ID, "user", umsg.ReplyToMessage.From.UserName,
			)
		}
	}

	if umsg.Quote != nil {
		log.Println("quote message",
			"text", replaceLineBreak(umsg.Quote.Text),
			"user id", umsg.From.ID, "user", umsg.From.UserName,
		)
	}

	argument := umsg.CommandArguments()

	if argument == "" {
		if umsg.Quote != nil {
			log.Println("use quote message", "text", replaceLineBreak(umsg.Quote.Text))
			argument = umsg.Quote.Text
		} else if umsg.ReplyToMessage != nil {
			if umsg.ReplyToMessage.Quote != nil {
				log.Println("use reply quote message", "text", replaceLineBreak(umsg.ReplyToMessage.Quote.Text))
				argument = umsg.ReplyToMessage.Quote.Text
			} else {
				log.Println("use reply message", "text", replaceLineBreak(umsg.ReplyToMessage.Text))
				argument = umsg.ReplyToMessage.Text
			}
		} else {
			return
		}
	}

	if _, err = Bot.Request(tgbotapi.NewChatAction(umsg.Chat.ID, "typing")); err != nil {
		log.Println("send typing status failed", "err", err)
	}

	switch command {
	case "gemma312b", "llama4scout17b16e":
		if edit {
			return
		}

		var result io.ReadCloser
		var err error

		if command == "llama4scout17b16e" {
			result, err = NewAI().LLama4Scout17b16eInstruct(Llama2_7bChatOptions{Prompt: argument})
		} else {
			result, err = NewAI().Gemma3_12b(Llama2_7bChatOptions{Prompt: argument})
		}
		if err != nil {
			log.Println("send message failed", "err", err)
			result = io.NopCloser(strings.NewReader(fmt.Sprintf(`data: {"response": "%s"}`, err.Error())))
		}

		ReturnByEventSource(result, umsg, argument)
		return
	}

	c := cache.New()

	var resptext string

	cachekey := &http.Request{
		Method: "GET", // only GET is supported cache
		URL: &url.URL{
			Scheme: "https",
			Host:   command + ".dict.worker.bot",
			Path:   argument,
		},
	}

	cache, err := c.Match(cachekey, &cache.MatchOptions{IgnoreMethod: true})
	if err != nil {
		log.Println("cache match failed", "err", err)
	} else {
		defer cache.Body.Close()
		data, err := io.ReadAll(cache.Body)
		if err == nil {
			log.Println("cache hit for", "command", command, "argument", argument)
			resptext = string(data)
		}
	}

	if resptext == "" {
		resp := translate(command, Args{Text: argument})
		if resp == nil {
			return
		}

		if len(resp) != 0 {
			resp = filterEmpty(resp)
		}

		if len(resp) == 0 {
			resp = []string{"not found"}
		}

		resptext = tgbotapi.EscapeText(tgbotapi.ModeHTML, strings.Join(resp, "\n\n"))

		err := c.Put(cachekey, &http.Response{
			Body: io.NopCloser(strings.NewReader(resptext)),
			Header: http.Header{
				"Cache-Control": []string{"max-age=86400"},
			},
		})
		if err != nil {
			log.Println("cache put failed", "err", err)
		}
	}

	msg := tgbotapi.NewMessage(umsg.Chat.ID, fmt.Sprintf("<blockquote expandable>%s</blockquote>", resptext))
	msg.ParseMode = tgbotapi.ModeHTML
	msg.LinkPreviewOptions.IsDisabled = true
	msg.ReplyParameters.MessageID = umsg.MessageID
	msg.ReplyMarkup = tgbotapi.NewInlineKeyboardMarkup(
		tgbotapi.NewInlineKeyboardRow(
			tgbotapi.NewInlineKeyboardButtonData("🗑️", "delete"),
			tgbotapi.NewInlineKeyboardButtonData("💾", fmt.Sprintf("save:%s", ifOr(len(argument) < 59, argument, ""))),
		),
	)
	if _, err = Bot.Request(msg); err != nil {
		log.Println("send message", "err", err)
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
	domain := "https://" + r.URL.Host
	if x := cloudflare.Getenv("worker_url"); x != "" {
		domain = x
	}

	wh, err := tgbotapi.NewWebhook(domain + "/tgbot")
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
		tgbotapi.BotCommand{Command: "gemma312b", Description: "translate by @cf/google/gemma-3-12b-it"},
		tgbotapi.BotCommand{Command: "llama4scout17b16e", Description: "translate by @cf/meta/llama-4-scout-17b-16e-instruct"},
		tgbotapi.BotCommand{Command: "ggtar_lang", Description: "google translate to tar_lang"},
		tgbotapi.BotCommand{Command: "delete", Description: "delete bot message"},
		tgbotapi.BotCommand{Command: "random", Description: "get random word for d1"},
	))
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	json.NewEncoder(w).Encode(map[string]any{
		"status": "ok",
		"result": resp,
		"domain": domain + "/tgbot",
	})
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

func filterEmpty(s []string) []string {
	result := make([]string, 0, len(s))
	for _, v := range s {
		if v != "" {
			result = append(result, v)
		}
	}
	return result
}

func deleteMessage(msg *tgbotapi.Message, callback bool) {
	if !callback {
		if msg.ReplyToMessage == nil {
			log.Println("delete reply to message is empty")
			return
		}

		if !msg.ReplyToMessage.From.IsBot {
			log.Println("delete reply to message is not bot")
			return
		}

		bu, err := Bot.GetMe()
		if err != nil {
			log.Println("get me", "err", err)
			return
		}

		if msg.ReplyToMessage.From.ID != bu.ID {
			log.Println("delete reply to message is not me")
			return
		}

		log.Println("delete reply to message", "chat_id", msg.ReplyToMessage.Chat.ID, "message_id", msg.ReplyToMessage.MessageID)
		msg = msg.ReplyToMessage
	} else {
		log.Println("delete callback message", "chat_id", msg.Chat.ID, "message_id", msg.MessageID)
	}

	dmsg := tgbotapi.NewDeleteMessage(msg.Chat.ID, msg.MessageID)
	if _, err := Bot.Request(dmsg); err != nil {
		log.Println("delete message", "err", err)
	}
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

func SendText(argument string, update *tgbotapi.Message, msgId int, text string) (tgbotapi.Message, error) {
	var msg tgbotapi.Chattable
	if msgId == 0 {
		m := tgbotapi.NewMessage(update.Chat.ID, text)
		m.ReplyParameters.MessageID = update.MessageID
		m.LinkPreviewOptions.IsDisabled = true
		m.ReplyMarkup = tgbotapi.NewInlineKeyboardMarkup(
			tgbotapi.NewInlineKeyboardRow(
				tgbotapi.NewInlineKeyboardButtonData("🗑️", "delete"),
				tgbotapi.NewInlineKeyboardButtonData("💾", "save:"),
			),
		)
		msg = m
	} else {
		msg = tgbotapi.NewEditMessageTextAndMarkup(update.Chat.ID, msgId, text, tgbotapi.NewInlineKeyboardMarkup(
			tgbotapi.NewInlineKeyboardRow(
				tgbotapi.NewInlineKeyboardButtonData("🗑️", "delete"),
				tgbotapi.NewInlineKeyboardButtonData("💾", "save:"),
			),
		))
	}

	rm, err := Bot.Send(msg)
	if err != nil {
		return rm, err
	}

	return rm, nil
}

func ifOr[T any](cond bool, a, b T) T {
	if cond {
		return a
	}
	return b
}

func editMessage(umsg *tgbotapi.Message, callData string, save bool) {
	msg := tgbotapi.NewEditMessageTextAndMarkup(umsg.Chat.ID, umsg.MessageID, umsg.Text, tgbotapi.NewInlineKeyboardMarkup(
		tgbotapi.NewInlineKeyboardRow(
			tgbotapi.NewInlineKeyboardButtonData("🗑️", "delete"),
			tgbotapi.NewInlineKeyboardButtonData(ifOr(save, "💾", "❌"), ifOr(save, "save:", "remove:")+callData),
		),
	))
	msg.ParseMode = tgbotapi.ModeHTML
	msg.LinkPreviewOptions.IsDisabled = true
	if _, err := Bot.Request(msg); err != nil {
		log.Println("edit message failed", "err", err)
	}
}

type callbackQuery struct {
	Command string
	Data    string
}

func (c callbackQuery) CallbackData() string {
	return c.Command + ":" + c.Data
}

func (c callbackQuery) IsSave() bool {
	return c.Command == "save"
}

func (c callbackQuery) IsRemove() bool {
	return c.Command == "remove"
}

func (c callbackQuery) IsDelete() bool {
	return c.Command == "delete"
}
