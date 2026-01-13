#

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

![screenshot](https://raw.githubusercontent.com/Asutorufa/hujiang_dictionary/refs/heads/rust/assets/images/web.png)

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
export CLOUDFLARE_D1_DATABASE_NAME=hujiang_dictionary

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
- CLOUDFLARE_D1_DATABASE_NAME=hujiang_dictionary  
    **cloudflare d1 database name**

build and deploy lambda

```shell
cargo lambda build --release --bin lambda
cargo lambda deploy --binary-name lambda hj-telegram-bot
```

init d1 table and register webhook

```shell
curl https://<lambda-url>/d1/create_table
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
WORKER_NAME="hj-dict" # cloudflare workers name
SCHEDULE="*/20 0-15 * * *" # cron schedule
```

custom openai api modal in `common/config/*.json`, which will auto merge all files

```json
{
    "openrouter": {
        "name": "openrouter",
        "base_url": "https://openrouter.ai/api/v1",
        "api_key": "sk-or-vx-************************",
        "reasoning": {
            "enabled": true
        },
        "models": [
            "openai/gpt-oss-20b:free",
            "deepseek/deepseek-chat-v3.1:free"
        ]
    },
    "gemini": {
        "name": "gemini",
        "base_url": "https://generativelanguage.googleapis.com/v1beta/openai",
        "api_key": "****************",
        "models": [
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "gemini-2.5-flash-lite",
            "gemini-2.0-flash",
            "gemini-2.0-flash-lite"
        ]
    }
}
```

init d1 table and register webhook

```shell
curl https://<workers-url>/d1/create_table
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
