#!/bin/bash
# Upload an asset file to R2.

set -e
set -o pipefail

CLOUDFLARE_R2_ASSETS_BUCKET_NAME=assets

: "${1:?"missing first argument [artifact name]"}"
: "${CLOUDFLARE_ASSETS_ACCOUNT_ID:?"missing environment variable"}"
: "${CLOUDFLARE_ASSETS_R2_ACCESS_KEY_ID:?"missing environment variable"}"
: "${CLOUDFLARE_ASSETS_R2_SECRET_ACCESS_KEY:?"missing environment variable"}"

bucket_name=$CLOUDFLARE_R2_ASSETS_BUCKET_NAME
endpoint_url="https://${CLOUDFLARE_ASSETS_ACCOUNT_ID}.r2.cloudflarestorage.com"

key="install/$1"

echo "Collecting the files to be uploaded…"

aws configure set default.s3.max_concurrent_requests 8
aws configure set default.s3.multipart_threshold 512MB
aws configure set default.s3.multipart_chunksize 128MB

s3_path="s3://${bucket_name}/${key}"
echo "Uploading to: $s3_path"

env -i \
    PATH="$PATH" \
    AWS_PAGER=0 \
    AWS_REGION=auto \
    AWS_ACCESS_KEY_ID="$CLOUDFLARE_ASSETS_R2_ACCESS_KEY_ID" \
    AWS_SECRET_ACCESS_KEY="$CLOUDFLARE_ASSETS_R2_SECRET_ACCESS_KEY" \
    aws --endpoint-url "$endpoint_url" s3 cp --no-progress - "$s3_path" < "$1"

env -i \
    PATH="$PATH" \
    AWS_PAGER=0 \
    AWS_REGION=auto \
    AWS_ACCESS_KEY_ID="$CLOUDFLARE_ASSETS_R2_ACCESS_KEY_ID" \
    AWS_SECRET_ACCESS_KEY="$CLOUDFLARE_ASSETS_R2_SECRET_ACCESS_KEY" \
    aws --endpoint-url "$endpoint_url" s3 ls "$s3_path" --human-readable

echo "Done."
