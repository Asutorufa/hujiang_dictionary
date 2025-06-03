package main

import (
	"flag"
	"log"
	"strconv"
	"strings"

	"github.com/Asutorufa/hujiang_dictionary/en"
	"github.com/Asutorufa/hujiang_dictionary/jp"
	"github.com/Asutorufa/hujiang_dictionary/kotobakku"
	"github.com/Asutorufa/hujiang_dictionary/kr"
	tgbotapi "github.com/OvyFlash/telegram-bot-api"
)

func main() {
	token := flag.String("t", "", "-t xxx, telegram bot token")
	id := flag.String("id", "", "-id [xx,xxx], user id")
	flag.Parse()

	if *token == "" {
		log.Panic("telegram bot token or instance name is empty")
	}

	bot, err := tgbotapi.NewBotAPI(*token)
	if err != nil {
		log.Panic(err)
	}

	var idMap map[int64]bool
	if *id != "" {
		idMap = make(map[int64]bool)
		for _, id := range strings.FieldsFunc(*id, func(r rune) bool { return r == ',' }) {
			i, err := strconv.ParseInt(id, 10, 64)
			if err != nil {
				log.Panic(err)
			}

			idMap[i] = true
		}

	}
	// bot.Debug = true

	log.Printf("Authorized on account %s", bot.Self.UserName)

	bot.Request(tgbotapi.NewSetMyCommands(
		tgbotapi.BotCommand{Command: "en", Description: "en"},
		tgbotapi.BotCommand{Command: "jpcn", Description: "jp -> cn"},
		tgbotapi.BotCommand{Command: "cnjp", Description: "cn -> jp"},
		tgbotapi.BotCommand{Command: "ktbk", Description: "コトバック"},
		tgbotapi.BotCommand{Command: "kr", Description: "kr"},
	))

	u := tgbotapi.NewUpdate(0)
	u.Timeout = 60

	updates := bot.GetUpdatesChan(u)

	for update := range updates {
		if update.Message == nil || (idMap != nil && !idMap[update.Message.From.ID]) {
			continue
		}

		// If we got a message
		log.Printf("[%s] %s", update.Message.From.UserName, update.Message.Text)

		argument := update.Message.CommandArguments()

		if argument == "" {
			continue
		}

		var resp []string
		switch update.Message.Command() {
		case "en":
			resp = en.FormatMarkdown(argument)
		case "jpcn":
			resp = jp.FormatMarkdown(argument)
		case "cnjp":
			resp = []string{jp.FormatCNString(argument)}
		case "ktbk":
			resp = []string{kotobakku.FormatString(argument)}
		case "kr":
			resp = []string{kr.FormatString(argument)}
		default:
			continue
		}

		for _, r := range resp {
			msg := tgbotapi.NewMessage(update.Message.Chat.ID, r)
			msg.ReplyParameters.MessageID = update.Message.MessageID
			bot.Send(msg)
		}

	}
}
