.PHONY: dev
dev:
	npx wrangler dev

.PHONY: build
build:
	go run github.com/syumai/workers/cmd/workers-assets-gen@v0.23.1
	tinygo build -panic=trap -gc=leaking -o ./build/app.wasm -target wasm -no-debug ./...

.PHONY: deploy
deploy:
	npx wrangler deploy