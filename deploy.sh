#!/bin/sh

set -x

if [ -f .env ]; then
    set -a;. ./.env;set +a
fi

if [ -f llm.json ]; then
    export CUSTOM_LLM_JSON_CONFIG=$(cat llm.json | base64 -w0)
fi

export D1_DATABASE_NAME="${D1_DATABASE_NAME:-dict}"
export WORKER_NAME="${WORKER_NAME:-hj-dict}"
export SCHEDULE="${SCHEDULE:-*/20 0-15 * * *}"
export CUSTOM_LLM_JSON_CONFIG="${CUSTOM_LLM_JSON_CONFIG:-$(echo "{}" | base64 -w0)}"

cat wrangler.toml | envsubst > wrangler.deploy.toml
npx wrangler deploy -c wrangler.deploy.toml
