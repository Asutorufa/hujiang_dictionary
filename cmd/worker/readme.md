## deploy

install wrangler

```shell
npm install
```

create wrangler.toml

```toml
name = "hj-dict"
main = "./build/worker.mjs"
compatibility_date = "2024-04-15"

[build]
command = "make build"

[vars]
telegram_token = "****:*****"
worker_url = "https://*****.workers.dev" # optional: if empty, use the reuqest domain
telegram_ids = "40xxxxxx,42xxxxx"
anki_telegram_id = "40xxxxxx" # send random word to the chat id when cron job run

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

then

```shell
make deploy
```
