#

- WIP
- Current only support telegram bot, please use golang version for full function
- Golang Version: [branch golang](https://github.com/Asutorufa/hujiang_dictionary/tree/golang)

- cloudflare api token need `d1` and `workers ai` permission.
- Either `d1 database id` or `d1 database name` must be provided. 

## build and run

```bash
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