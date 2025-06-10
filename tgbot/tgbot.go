package tgbot

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"strconv"
	"strings"

	"github.com/Asutorufa/hujiang_dictionary/en"
	"github.com/Asutorufa/hujiang_dictionary/google"
	"github.com/Asutorufa/hujiang_dictionary/jp"
	"github.com/Asutorufa/hujiang_dictionary/kotobakku"
	"github.com/Asutorufa/hujiang_dictionary/kr"
	"github.com/Asutorufa/hujiang_dictionary/weblio"
	tgbotapi "github.com/OvyFlash/telegram-bot-api"
)

type Cache interface {
	Get(command string) (string, bool)
	Set(command string, result string)
}

type DB interface {
	Save(word string, explain string) error
	Random() (string, string, error)
	Remove(word string) error
}

var (
	telegramIds     string
	workerUrl       string
	tgbot           *tgbotapi.BotAPI
	externalCommand func(cmd string, args Args) bool
	cache           Cache
	db              DB
)

func Handler(
	telegram_ids string,
	token string,
	client tgbotapi.HTTPClient,
	c Cache,
	d DB,
) func(r io.Reader) (any, error) {
	tgbot = &tgbotapi.BotAPI{
		Token:  token,
		Client: client,
	}

	tgbot.SetAPIEndpoint(tgbotapi.APIEndpoint)
	// tgbot.Debug = true
	telegramIds = telegram_ids
	cache = c
	db = d

	return handler
}

func handler(r io.Reader) (any, error) {
	var update tgbotapi.Update
	err := json.NewDecoder(r).Decode(&update)
	if err != nil {
		return nil, err
	}

	var umsg *tgbotapi.Message
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
	} else {
		return nil, fmt.Errorf("unsupported update")
	}

	authorized := false
	for _, id := range strings.FieldsFunc(telegramIds, func(r rune) bool { return r == ',' }) {
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
			return nil, fmt.Errorf("callback message is nil")
		}

		switch callback.Command {
		case "delete":
			deleteMessage(update.CallbackQuery.Message, true)
			return nil, nil
		case "save", "remove":
			var word string = callback.Data

			if word == "" {
				if umsg.ReplyToMessage == nil {
					return nil, fmt.Errorf("callback reply to message is nil")
				}

				word = umsg.ReplyToMessage.CommandArguments()
				if word == "" {
					return nil, fmt.Errorf("callback reply to message is empty")
				}
			}

			if db == nil {
				return nil, fmt.Errorf("db is nil")
			}

			if callback.IsSave() {
				err = db.Save(word, update.CallbackQuery.Message.Text)
			} else {
				err = db.Remove(word)
			}
			if err != nil {
				log.Println(callback.Command, "word failed", "word", word, "err", err)
			} else {
				log.Println(callback.Command, "word success", "word", word)
				editMessage(umsg, callback.Data, !callback.IsSave())
			}

			return nil, nil
		}
	}

	if umsg == nil || !authorized {
		return nil, nil
	}

	command := umsg.Command()

	switch command {
	case "delete":
		deleteMessage(umsg, false)
		return nil, nil
	case "random":
		if db == nil {
			return nil, fmt.Errorf("db is nil")
		}

		word, explain, err := db.Random()
		if err != nil {
			return nil, fmt.Errorf("get random word failed: %w", err)
		}

		msg := tgbotapi.NewMessage(umsg.Chat.ID, fmt.Sprintf(`<b>%s</b>
<tg-spoiler><blockquote expandable>%s</blockquote></tg-spoiler>
`, tgbotapi.EscapeText(tgbotapi.ModeHTML, word),
			tgbotapi.EscapeText(tgbotapi.ModeHTML, explain)))
		msg.ParseMode = tgbotapi.ModeHTML
		msg.LinkPreviewOptions.IsDisabled = true
		msg.ReplyParameters.MessageID = umsg.MessageID
		_, err = tgbot.Send(msg)
		if err != nil {
			return nil, fmt.Errorf("send message failed: %w", err)
		}

		return nil, nil
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
			return nil, nil
		}
	}

	if _, err = tgbot.Request(tgbotapi.NewChatAction(umsg.Chat.ID, "typing")); err != nil {
		return nil, fmt.Errorf("send typing status failed: %w", err)
	}

	if externalCommand != nil && externalCommand(command, Args{Text: argument}) {
		return nil, nil
	}

	var resptext string

	if cache != nil {
		cache, ok := cache.Get(command)
		if ok {
			log.Println("cache hit for", "command", command, "argument", argument)
			resptext = cache
		}
	}

	if resptext == "" {
		resp := translate(command, Args{Text: argument})
		if resp == nil {
			return nil, nil
		}

		if len(resp) != 0 {
			resp = filterEmpty(resp)
		}

		if len(resp) == 0 {
			resp = []string{"not found"}
		}

		resptext = tgbotapi.EscapeText(tgbotapi.ModeHTML, strings.Join(resp, "\n\n"))

		if cache != nil {
			cache.Set(command, resptext)
		}
	}

	msg := tgbotapi.NewMessage(
		umsg.Chat.ID,
		fmt.Sprintf("<blockquote expandable>%s</blockquote>", resptext))
	msg.ParseMode = tgbotapi.ModeHTML
	msg.LinkPreviewOptions.IsDisabled = true
	msg.ReplyParameters.MessageID = umsg.MessageID
	msg.ReplyMarkup = tgbotapi.NewInlineKeyboardMarkup(
		tgbotapi.NewInlineKeyboardRow(
			tgbotapi.NewInlineKeyboardButtonData("🗑️", "delete"),
			tgbotapi.NewInlineKeyboardButtonData("💾", fmt.Sprintf("save:%s", ifOr(len(argument) < 59, argument, ""))),
		),
	)
	if _, err = tgbot.Request(msg); err != nil {
		return nil, fmt.Errorf("send message failed: %w", err)
	}

	return nil, nil
}

func register(w http.ResponseWriter, r *http.Request) {
	domain := "https://" + r.URL.Host
	if workerUrl != "" {
		domain = workerUrl
	}

	wh, err := tgbotapi.NewWebhook(domain + "/tgbot")
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	_, err = tgbot.Request(wh)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	resp, err := tgbot.Request(tgbotapi.NewSetMyCommands(
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

		bu, err := tgbot.GetMe()
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
	if _, err := tgbot.Request(dmsg); err != nil {
		log.Println("delete message", "err", err)
	}
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

	rm, err := tgbot.Send(msg)
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
	msg := tgbotapi.NewEditMessageTextAndMarkup(
		umsg.Chat.ID,
		umsg.MessageID,
		fmt.Sprintf("<blockquote expandable>%s</blockquote>", tgbotapi.EscapeText(tgbotapi.ModeHTML, umsg.Text)),
		tgbotapi.NewInlineKeyboardMarkup(
			tgbotapi.NewInlineKeyboardRow(
				tgbotapi.NewInlineKeyboardButtonData("🗑️", "delete"),
				tgbotapi.NewInlineKeyboardButtonData(ifOr(save, "💾", "❌"), ifOr(save, "save:", "remove:")+callData),
			),
		))
	msg.ParseMode = tgbotapi.ModeHTML
	msg.LinkPreviewOptions.IsDisabled = true
	if _, err := tgbot.Request(msg); err != nil {
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
