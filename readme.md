# DictDeck

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

Support run telegram at local, lambda and cloudflare workers.

![screenshot](https://raw.githubusercontent.com/Asutorufa/hujiang_dictionary/rust/assets/images/telegram.png)

### service

```shell
cargo build --release

export TELOXIDE_TOKEN=12312313:sadsadasda
export MAINTAINER_ID=312321312
export ALLOW_USERS=312321312,232133424,123131243
export CLOUDFLARE_ACCOUNT_ID=dksaodjasopdjpadjapd
export CLOUDFLARE_API_TOKEN=dkapdpaksdpaspdnsknszcl
export CLOUDFLARE_D1_DATABASE_ID=231331-adae-3123-vdfsf-1313adssaeqewq
export CLOUDFLARE_D1_DATABASE_NAME=dictdeck

./target/release/tg
```

### lambda

set blow env in lambda

- TELOXIDE_TOKEN=12312313:sadsadasda  
    **telegram bot token**
- MAINTAINER_ID=312321312  
    **telegram user id**
- ALLOW_USERS=312321312,232133424,123131243  
    **allow telegram user id**
- CLOUDFLARE_ACCOUNT_ID=dksaodjasopdjpadjapd  
    **cloudflare account id**
- CLOUDFLARE_API_TOKEN=dkapdpaksdpaspdnsknszcl  
    **cloudflare api token**
- CLOUDFLARE_D1_DATABASE_ID=231331-adae-3123-vdfsf-1313adssaeqewq  
    **cloudflare d1 database id**
- CLOUDFLARE_D1_DATABASE_NAME=dictdeck  
    **cloudflare d1 database name**

build and deploy lambda

```shell
cargo lambda build --release --bin lambda
cargo lambda deploy --binary-name lambda hj-telegram-bot
```

register webhook

```shell
curl https://<lambda-url>/tgbot/register
```

## cloudflare workers

set wrangler config

```shell
vim wrangler.toml
# build and deploy
cargo install worker-build
npx wrangler deploy
```

`wrangler.toml` config example

```toml
D1_DATABASE_NAME=dict # d1 database name
D1_DATABASE_ID="57ccd046-bd5c-42a3-90a3-21da43bc119d" # d1 database id
TELEGRAM_TOKEN="****:*****" # telegram bot token
ALLOW_USERS="12345678,-23456789,34567890" # allow telegram user id, split by comma
MAINTAINER_ID="12345678" # send random word to the chat id when cron job run
WORKER_NAME="dictdeck" # cloudflare workers name
SCHEDULE="*/20 0-15 * * *" # cron schedule
```

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
cargo install worker-build
```

Deploy Command

```shell
export PATH="$HOME/.cargo/bin:$PATH"
npx wrangler deploy
```
