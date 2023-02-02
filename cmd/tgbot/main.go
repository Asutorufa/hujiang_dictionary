package main

import (
	"flag"
	"log"

	"github.com/Asutorufa/hujiang_dictionary/en"
	"github.com/Asutorufa/hujiang_dictionary/jp"
	"github.com/Asutorufa/hujiang_dictionary/kotobakku"
	"github.com/Asutorufa/hujiang_dictionary/kr"
	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api/v5"
)

func main() {
	token := flag.String("t", "", "-t xxx, telegram bot token")
	id := flag.Int64("id", 0, "-id xx, user id")
	flag.Parse()

	if *token == "" {
		log.Panic("telegram bot token or instance name is empty")
	}

	bot, err := tgbotapi.NewBotAPI(*token)
	if err != nil {
		log.Panic(err)
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
		if update.Message == nil || (*id != 0 && update.Message.From.ID != *id) {
			continue
		}

		// If we got a message
		log.Printf("[%s] %s", update.Message.From.UserName, update.Message.Text)

		argument := update.Message.CommandArguments()

		if argument == "" {
			continue
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
			continue
		}

		if resp == "" {
			continue
		}

		msg := tgbotapi.NewMessage(update.Message.Chat.ID, resp)
		msg.ReplyToMessageID = update.Message.MessageID

		bot.Send(msg)
	}
}
