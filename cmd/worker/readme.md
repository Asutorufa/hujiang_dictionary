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
worker_url = "https://*****.workers.dev"
telegram_ids = "40xxxxxx,42xxxxx"
```


then

```shell
make deploy
```
