#

- cloudflare api token need `d1` and `workers ai` permission.
- Either `d1 database id` or `d1 database name` must be provided.

## build and run

```bash
cargo build --release
./target/release/hj jc こんにちは
```

## cli

- `jc <word>` - Japanese to Chinese
- `cj <word>` - Chinese to Japanese
- `en <word>` - English to Japanese
- `weblio <word>` - weblio
- `ktbk <word>` - コトバック
- `google <target> <words>` - Google Translate, eg: google en こんにちは

Example:

```shell
./target/release/hj jc こんにちは
./target/release/hj cj 你好
./target/release/hj en hello
./target/release/hj en 你好
./target/release/hj weblio こんにちは
./target/release/hj ktbk 子供
./target/release/hj google ja Hello world!
```

![screenshot](https://raw.githubusercontent.com/Asutorufa/hujiang_dictionary/rust/assets/images/image.png)

## telegram bot

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

## lambda

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

```bash
cargo lambda build --release --bin lambda
cargo lambda deploy --binary-name lambda hj-telegram-bot
```

init d1 table and register webhook

```bash
curl https://<lambda-url>/d1/create_table
curl https://<lambda-url>/tgbot/register
```


## cloudflare workers

create wrangler config

```shell
cd worker
vim wrangler.toml
```

```toml
name = "hj-rust"
main = "build/worker/shim.mjs"
compatibility_date = "2023-03-22"

[build]
command = "worker-build --release"

[vars]
TELEGRAM_TOKEN = "****:*****"
ALLOW_USERS = "42xxxxx"
MAINTAINER_ID = "40xxxxxx" # send random word to the chat id when cron job run

[ai]
binding = "AI"

[observability.logs]
enabled = true

[[d1_databases]]
binding = "DB"
database_name = "dict"
database_id = "xxx-xxx-xxxx"

[triggers]
crons = ["0 * * * *"]
```

build/dev/deploy

```shell
cargo install worker-build
npx wrangler build
npx wrangler dev
npx wrangler deploy
```

init d1 table and register webhook

```bash
curl https://<workers-url>/d1/create_table
curl https://<workers-url>/tgbot/register
```

## Others

- Golang Version: [branch golang](https://github.com/Asutorufa/hujiang_dictionary/tree/golang)
