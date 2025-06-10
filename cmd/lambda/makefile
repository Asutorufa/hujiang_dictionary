.PHONY: all
all:
	CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -ldflags="-s -w -buildid=" -trimpath -v -o bootstrap .
	zip bootstrap.zip bootstrap
