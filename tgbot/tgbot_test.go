package tgbot

import (
	"os"
	"strconv"
	"testing"

	tgbotapi "github.com/OvyFlash/telegram-bot-api"
	"github.com/go-json-experiment/json"
	"github.com/stretchr/testify/require"
)

func TestTgbot(t *testing.T) {
	idstr, err := os.ReadFile(".tgid")
	require.NoError(t, err)
	id, err := strconv.ParseInt(string(idstr), 10, 64)
	require.NoError(t, err)

	u := tgbotapi.Update{
		Message: &tgbotapi.Message{
			From: &tgbotapi.User{
				ID: id,
			},
			MessageID: 0,
			Chat: tgbotapi.Chat{
				ID: id,
			},
			Entities: []tgbotapi.MessageEntity{
				{
					Type:   "bot_command",
					Length: 3,
				},
			},
			Text: " en@hello",
		},
	}

	t.Log(u.Message.Command(), u.Message.CommandArguments())

	json.MarshalWrite(os.Stdout, u)
}
