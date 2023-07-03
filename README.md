# wormhole3_account_binding
Wormhole3 Account Bindings Contract on NEAR

## Environment Requirements

- Rust v1.66 (e.g. v1.66.1)
- Node v16 (e.g. v16.18.1)

## Set Up Project

```bash
npm i
```

## Build Contract

```bash
make build
```

## Test Contract

Run both unit test and integration test

```bash
make test
```

## Deploy Contract

Deploy and initialize contract, and add default manager account.

Specify root account with `ROOT_ACCOUNT`.

```bash
# prepare accounts and deploy contract
ROOT_ACCOUNT=wormhole3.testnet make deploy
```
