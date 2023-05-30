# Running with DynamoDB Local

## Assets

Assets in the API repo need to be built:

```sh
RELEASE_FLAG="--dev" GATEWAY_FEATURES=local,dynamodb ./scripts/dev/build-cli-assets.sh
```

## Set up

DynamoDB Local must be running:

```sh
docker run -p 8000:8000 --rm --name dynamodb -d amazon/dynamodb-local
```

To execute the CLI, one must create a table:

## Preparing the table for the CLI

```sh
export AWS_ACCESS_KEY_ID="fakeMyKeyId"
export AWS_SECRET_ACCESS_KEY="fakeSecretAccessKey"
dy -r local admin create table gateway --keys __pk __sk
dy -r local admin create index gsi1 -t gateway --keys __gsi1pk __gsi1sk
dy -r local admin create index gsi2 -t gateway --keys __gsi2pk __gsi2sk
```

## Running the CLI

```sh
export AWS_ACCESS_KEY_ID="fakeMyKeyId"
export AWS_SECRET_ACCESS_KEY="fakeSecretAccessKey"
export DYNAMODB_REGION="custom:http://localhost:8000"
export DYNAMODB_TABLE_NAME=gateway

rm -rf ~/.grafbase # version is the same so the cache can contain the wrong wasm variant

cargo run -p grafbase --features=dynamodb -- -t 2 dev
```

## Running the tests

Testing tables are created and deleted on the fly.

```sh
export AWS_ACCESS_KEY_ID="fakeMyKeyId"
export AWS_SECRET_ACCESS_KEY="fakeSecretAccessKey"
export DYNAMODB_REGION="custom:http://localhost:8000"

cargo nextest run --features=dynamodb  -P ci
```
