.PHONY: dev
dev:
	npx wrangler dev

.PHONY: build
build:
	go run github.com/syumai/workers/cmd/workers-assets-gen@latest
	# tinygo build -panic=trap -gc=leaking -o ./build/app.wasm -target wasm -no-debug ./...
	tinygo build -o ./build/app.wasm -target wasm -no-debug ./...

.PHONY: deploy
deploy:
	npx wrangler deploy
