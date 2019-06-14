# Slaps Roof Of Wallet

This is a not-at-all-useful GUI wallet for Lightning built with [lnd-rust](https://github.com/LightningPeach/lnd-rust) and [druid](https://github.com/xi-editor/druid).

## Setup

You need a .env file that looks something like this:

```sh
export LND_GRPC_URL="127.0.0.1:10009"
export LND_READ_MACAROON="/path/to/lnd/readonly.macaroon"
export LND_INVOICE_MACAROON="/path/to/lnd/invoice.macaroon"
export LND_TLS_CERT="/path/to/lnd/tls.cert"
```

Then run `source .env` in your terminal to set those enviroment variables for your current terminal.

For the build step you'll need [protoc](https://github.com/protocolbuffers/protobuf/releases) on your path, along with [lnd installed on your $GOPATH](https://github.com/lightningnetwork/lnd/blob/master/docs/INSTALL.md).

On Mac you'll also need Cairo (`brew install cairo`).

