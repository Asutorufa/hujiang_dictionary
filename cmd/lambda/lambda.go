package main

import (
	"context"
	"encoding/base64"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"

	"github.com/Asutorufa/hujiang_dictionary/tgbot"
	"github.com/aws/aws-lambda-go/events"
	"github.com/aws/aws-lambda-go/lambda"
	"github.com/go-json-experiment/json"
)

func handleRequest(ctx context.Context, request events.LambdaFunctionURLRequest) (events.LambdaFunctionURLResponse, error) {
	switch request.RawPath {
	case "/tgbot":
		handler := tgbot.Handler(
			os.Getenv("telegram_ids"),
			os.Getenv("telegram_token"),
			http.DefaultClient,
			nil,
			nil,
		)
		var body io.Reader = strings.NewReader(request.Body)

		if request.IsBase64Encoded {
			body = base64.NewDecoder(base64.StdEncoding, body)
		}

		resp, err := handler(body)
		if err != nil {
			return events.LambdaFunctionURLResponse{
				StatusCode: http.StatusInternalServerError,
				Body:       err.Error(),
			}, nil
		} else {
			return events.LambdaFunctionURLResponse{
				StatusCode: http.StatusOK,
				Body:       fmt.Sprint(resp),
			}, nil
		}

	default:
	}

	if request.IsBase64Encoded {
		body, err := base64.StdEncoding.DecodeString(request.Body)
		if err == nil {
			request.Body = string(body)
		}
	}

	data, err := json.Marshal(request)
	if err != nil {
		data = []byte(err.Error())
	}
	return events.LambdaFunctionURLResponse{Body: string(data), StatusCode: 200}, nil
}

func main() {
	// Make the handler available for Remote Procedure Call by AWS Lambda
	lambda.Start(handleRequest)
}
