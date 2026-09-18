# DictDeck

<p align="center">
  <img src="./assets/dictdeck.svg" alt="DictDeck" width="128" />
</p>

DictDeck is a self-hosted AI dictionary, translator, and vocabulary flashcard app.
It supports multiple dictionary sources, LLM-powered explanations, saved words,
review cards, Telegram bot workflows, and Cloudflare Workers deployment.

[![Deploy to Cloudflare](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/Asutorufa/hujiang_dictionary)

- cloudflare api token need `d1` and `workers ai` permission.
- Either `d1 database id` or `d1 database name` must be provided.

## build and run

```bash
cargo build --release
./target/release/hj jc こんにちは
```

## web

see [telegram bot](#telegram-bot)

https://github.com/user-attachments/assets/19575332-0906-4654-a173-b8c390e0baf4

![screenshot](https://raw.githubusercontent.com/Asutorufa/hujiang_dictionary/refs/heads/rust/assets/images/web.png)

## browser extension

The repository also includes a browser extension for translating selected text from any page.

Local build:

```bash
cd web
npm install
npm run build:extension
```

Build outputs:

- `web/extension-dist/chrome` - unpacked Chrome extension
- `web/extension-dist/firefox` - unpacked Firefox extension
- `web/extension-dist/safari` - generated Safari project
- `web/extension-dist/safari-app` - built Safari macOS app

Notes:

- Safari packaging is built on macOS and requires Xcode / Command Line Tools.
- The Safari app is generated so the extension can be enabled through Safari on macOS.

## ci artifacts

The GitHub Actions workflow at [`.github/workflows/rust.yaml`](./.github/workflows/rust.yaml) builds and uploads extension artifacts on macOS.

Uploaded artifacts:

- `dictdeck-extension-chrome.zip`
- `dictdeck-extension-firefox.zip`
- `dictdeck-extension-safari-project.zip`
- `dictdeck-extension-safari-app.zip`

These files are uploaded as workflow artifacts in CI, and the release workflow also includes them in release assets.

## cli

- `jc <word>` - Japanese to Chinese
- `cj <word>` - Chinese to Japanese
- `en <word>` - English to Japanese
- `weblio <word>` - weblio
- `ktbk <word>` - コトバンク
- `google <target> <words>` - Google Translate, eg: google en こんにちは

Example:

```shell
./target/release/hj jc こんにちは
./target/release/hj cj 你好
./target/release/hj kr 안녕하세요
./target/release/hj en hello
./target/release/hj en 你好
./target/release/hj weblio こんにちは
./target/release/hj ktbk 子供
./target/release/hj google ja Hello world!
```

![screenshot](https://raw.githubusercontent.com/Asutorufa/hujiang_dictionary/rust/assets/images/image.png)

## telegram bot

Telegram bot, HTTP API, and scheduled jobs run on Cloudflare Workers.

![screenshot](https://raw.githubusercontent.com/Asutorufa/hujiang_dictionary/rust/assets/images/telegram.png)

## cloudflare workers

set wrangler config

```shell
vim wrangler.toml
# set the production JWT signing secret outside source control
npx wrangler secret put AUTH_SECRET
# build and deploy
cargo install worker-build --version 0.8.6
npx wrangler deploy
```

`wrangler.toml` config example

```toml
WORKER_NAME="dictdeck" # cloudflare workers name
SCHEDULE="*/20 0-15 * * *" # cron schedule
```

### MCP dictionary server

The Worker exposes a Streamable HTTP MCP endpoint at `/mcp`. In the web settings page, generate a token with `dictionary:read` and/or `dictionary:write` permission. The plaintext token is shown only once; send it as a Bearer token:

```shell
curl https://<workers-url>/mcp \
  -H 'Authorization: Bearer mcp_...' \
  -H 'Accept: application/json, text/event-stream' \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
```

Read access provides `dictionary.search` and `dictionary.get`. Write access adds save, rename, delete, priority, and preview/apply batch-change tools. Set `MCP_ALLOWED_ORIGINS` to a comma-separated origin allowlist when browser-based MCP clients are used.

register webhook

```shell
curl https://<workers-url>/tgbot/register
```

If use workers CI/CD, you can add following script in  `Build Command` and `Deploy Command`

Build Command

```shell
git clone -b react https://github.com/Asutorufa/hujiang_dictionary.git react
cd react && npm install && npm run build && cd ..
cp -r react/out web/out
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > rustup.sh
sh rustup.sh -y
export PATH="$HOME/.cargo/bin:$PATH"
cargo install worker-build --version 0.8.6
```

Deploy Command

```shell
export PATH="$HOME/.cargo/bin:$PATH"
npx wrangler deploy
```
