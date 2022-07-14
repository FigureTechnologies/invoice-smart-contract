# Invoice Smart Contact

This contract facilitates the transfer of restricted coin between addresses.

## Status

[![Latest Release][release-badge]][release-latest]
[![Apache 2.0 License][license-badge]][license-url]
[![Code Coverage][codecov-badge]][codecov-report]

[license-badge]: https://img.shields.io/badge/License-Apache_2.0-blue.svg
[license-url]: https://github.com/FigureTechnologies/invoice-smart-contract/blob/main/LICENSE
[release-badge]: https://img.shields.io/github/tag/FigureTechnologies/invoice-smart-contract.svg
[release-latest]: https://github.com/FigureTechnologies/invoice-smart-contract/releases/latest
[codecov-badge]: https://codecov.io/gh/FigureTechnologies/invoice-smart-contract/branch/main/graph/badge.svg
[codecov-report]: https://codecov.io/gh/FigureTechnologies/invoice-smart-contract

## Background

As a USDX merchant, you want to be able to use a blockchain-based invoice creation and payment processing system without a centralized
authority.

This contract allows a merchant to create and receive payments by recording an invoice in smart contract state, and direct payments
to a recipient address.

## Assumptions

This README assumes you are familiar with writing and deploying smart contracts to the
[provenance](https://docs.provenance.io/) blockchain.
See the `provwasm` [tutorial](https://github.com/provenance-io/provwasm/blob/main/docs/tutorial/01-overview.md)
for details.

### [Provenance Testnet](https://github.com/provenance-io/testnet) Deployments
#### [pio-testnet-1](https://github.com/provenance-io/testnet/tree/main/pio-testnet-1)

| Contract Version | Code ID |
|------------------|---------|
| 0.1.0            | x       |

## Blockchain Quickstart

Checkout provenance v1.11.0, install the `provenanced` command and start a 4-node localnet.

```bash
git clone https://github.com/provenance-io/provenance.git
cd provenance && git checkout v1.11.0
make install
make localnet-start
```

## Accounts

Accounts need to be set up for example users and marker admins.

Admin

```bash
provenanced keys add admin \
    --home build/node0 --keyring-backend test --testnet --hd-path "44'/1'/0'/0/0" --output json | jq

{
  "name": "admin",
  "type": "local",
  "address": "tp10nnm70y8zc5m8yje5zx5canyqq639j3ph7mj8p",
  "pubkey": "tppub1addwnpepqf4feq9n484c6tvpcugkp0l78mffld8aphq8wqehx53pekcf2l5pkuajggq",
  "mnemonic": "seminar tape camp attract student make hollow pyramid obtain bamboo exit donate dish drip text foil news film assist access pride decline reason lonely"
}
```

Merchant

```bash
provenanced keys add merchant \
    --home build/node0 --keyring-backend test --testnet --hd-path "44'/1'/0'/0/0" --output json | jq

{
  "name": "merchant",
  "type": "local",
  "address": "tp1m4arun5y9jcwkatq2ey9wuftanm5ptzsg4ppfs",
  "pubkey": "tppub1addwnpepqgw8y7dpx4xmlaun5u55qrq4e05jtul6nu94afq3tvr7e8d4xx6ujzf79jz",
  "mnemonic": "immense ordinary august exclude loyal expire install tongue ski bounce sock buffalo range begin glory inch index float medal kid empty wheel badge find"
}
```

Recipient

```bash
provenanced keys add recipient \
    --home build/node0 --keyring-backend test --testnet --hd-path "44'/1'/0'/0/0" --output json | jq

{
  "name": "recipient",
  "type": "local",
  "address": "tp1cxjkp6sxregvhqfqc74ythsha6g00dnry9ef6m",
  "pubkey": "{\"@type\":\"/cosmos.crypto.secp256k1.PubKey\",\"key\":\"A4wAr0yMRN09GUYRCxV2xMliAnTGKvFksLtRFJ8O8Mnl\"}",
  "mnemonic": "provide gate cute aspect opinion toast sport habit join want gold retreat option arrest roof idle outer olive leisure portion cycle horror subway ghost"
}
```

Customer

```bash
provenanced keys add customer \
    --home build/node0 --keyring-backend test --testnet --hd-path "44'/1'/0'/0/0" --output json | jq

{
  "name": "customer",
  "type": "local",
  "address": "tp15nauudez3yvrma9mfve7t9hnnnlkgc7fwps85d",
  "pubkey": "{\"@type\":\"/cosmos.crypto.secp256k1.PubKey\",\"key\":\"AlOF+u9+kMmP3mLlny+u2S7WBgDnJqJOwzJVXCFJZOgI\"}",
  "mnemonic": "develop glory absurd glory march valve hunt barely inform luxury ahead miss eye minimum assault meat pair shoot magic develop argue exact believe faint"
}
```

If you want to use the addresses from this document, use the mnemonics above to restore the keys locally.

For example:

```bash
provenanced keys add admin --recover \
    --home build/node0 --keyring-backend test --testnet --hd-path "44'/1'/0'/0/0"
```

## Fee Payment

Fund the example accounts with `nhash` to pay network fees.

```bash
provenanced tx bank send \
    $(provenanced keys show -a node0 --home build/node0 --keyring-backend test --testnet) \
    $(provenanced keys show -a admin --home build/node0 --keyring-backend test --testnet) \
    100000000000nhash \
    --from node0 \
    --keyring-backend test \
    --home build/node0 \
    --chain-id chain-local \
    --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
    --broadcast-mode block \
    --yes \
    --testnet -o json | jq
```

```bash
provenanced tx bank send \
    $(provenanced keys show -a node0 --home build/node0 --keyring-backend test --testnet) \
    $(provenanced keys show -a merchant --home build/node0 --keyring-backend test --testnet) \
    100000000000nhash \
    --from node0 \
    --keyring-backend test \
    --home build/node0 \
    --chain-id chain-local \
    --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
    --broadcast-mode block \
    --yes \
    --testnet -o json | jq
```

```bash
provenanced tx bank send \
    $(provenanced keys show -a node0 --home build/node0 --keyring-backend test --testnet) \
    $(provenanced keys show -a customer --home build/node0 --keyring-backend test --testnet) \
    100000000000nhash \
    --from node0 \
    --keyring-backend test \
    --home build/node0 \
    --chain-id chain-local \
    --gas auto --gas-prices 1905nhash --gas-adjustment 2 \
    --broadcast-mode block \
    --yes \
    --testnet -o json | jq
```

## Marker creation

Create an unrestricted marker representing USDX.

```bash
provenanced tx marker new "100000usdx.c" \
  --type COIN \
  --from admin \
  --home build/node0 --keyring-backend test \
  --chain-id chain-local \
  --gas auto --gas-prices 1905nhash --gas-adjustment 1.3 \
  --yes \
  --testnet -o json | jq
```

Grant marker admin access to `admin`

```bash
provenanced tx marker grant $(provenanced keys show -a admin --home build/node0 --keyring-backend test --testnet) usdx.c admin,withdraw,burn,mint \
  --from admin \
  --home build/node0 --keyring-backend test \
  --chain-id chain-local \
  --gas auto --gas-prices 1905nhash --gas-adjustment 1.3 \
  --yes \
  --testnet -o json | jq
```

Finalize the marker

```bash
provenanced tx marker finalize usdx.c \
  --from admin \
  --home build/node0 --keyring-backend test \
  --chain-id chain-local \
  --gas auto --gas-prices 1905nhash --gas-adjustment 1.3 \
  --yes \
  --testnet -o json | jq
```

Activate the marker

```bash
provenanced tx marker activate usdx.c \
  --from admin \
  --home build/node0 --keyring-backend test \
  --chain-id chain-local \
  --gas auto --gas-prices 1905nhash --gas-adjustment 1.3 \
  --yes \
  --testnet -o json | jq
```

Now distribute shares of `usdx.c` coin to `customer` for use.

```bash
provenanced tx marker withdraw usdx.c 100000usdx.c $(provenanced keys show -a customer --home build/node0 --keyring-backend test --testnet)  \
  --from admin \
  --home build/node0 --keyring-backend test \
  --chain-id chain-local \
  --gas auto --gas-prices 1905nhash --gas-adjustment 1.3 \
  --yes \
  --testnet -o json | jq
```

## Store the Wasm

Store the optimized smart contract Wasm on-chain. This assumes you've copied `artifacts/invoice`
to the provenance root dir (ie where the localnet was started from).

```bash
provenanced tx wasm store invoice.wasm \
  --from admin \
  --home build/node0 --keyring-backend test \
  --chain-id chain-local \
  --gas auto --gas-prices 1905nhash --gas-adjustment 1.3 \
  --testnet \
  --yes -o json | jq
```

## Instantiate the contract

Instantiate the contract using the `code_id` returned from storing the Wasm. Note the contract address returned.

```bash
provenanced tx wasm instantiate 1 \
  '{"denom":"usdx.c","recipient":"tp1cxjkp6sxregvhqfqc74ythsha6g00dnry9ef6m","business_name":"Shoe Co, LLC"}' \
  --label invoice1 \
  --admin $(provenanced keys show -a merchant --home build/node0 --keyring-backend test --testnet) \
  --from merchant \
  --home build/node0 --keyring-backend test \
  --chain-id chain-local \
  --gas auto --gas-prices 1905nhash --gas-adjustment 1.3 \
  --testnet \
  --yes -o json | jq

{
  "msg_index": 0,
  "log": "",
  "events": [
    {
      "type": "instantiate",
      "attributes": [
        {
          "key": "_contract_address",
          "value": "tp153r9tg33had5c5s54sqzn879xww2q2egektyqnpj6nwxt8wls70qrv2qq2"
        },
        {
          "key": "code_id",
          "value": "6"
        }
      ]
    },
    {
      "type": "message",
      "attributes": [
        {
          "key": "action",
          "value": "/cosmwasm.wasm.v1.MsgInstantiateContract"
        },
        {
          "key": "module",
          "value": "wasm"
        },
        {
          "key": "sender",
          "value": "tp1m4arun5y9jcwkatq2ey9wuftanm5ptzsg4ppfs"
        }
      ]
    }
  ]
}
```

## Contract execution example

### Add Invoice

`merchant` first needs to create an invoice with the smart contract.

```bash
provenanced tx wasm execute tp153r9tg33had5c5s54sqzn879xww2q2egektyqnpj6nwxt8wls70qrv2qq2 \
    '{"add_invoice":{"id":"63069195-bc51-41bd-80d7-0ab84b98e283", "amount":"10000", "description": "Air Jordan High Black Red"}}' \
    --from merchant \
    --home build/node0 --keyring-backend test \
    --chain-id chain-local \
    --gas auto --gas-prices 1905nhash --gas-adjustment 1.3 \
    --testnet \
    --yes -o json | jq
```

The invoice should now be in the smart contract state and be queryable.

```bash
provenanced query wasm contract-state smart tp153r9tg33had5c5s54sqzn879xww2q2egektyqnpj6nwxt8wls70qrv2qq2 \
    '{"get_invoice":{"id":"63069195-bc51-41bd-80d7-0ab84b98e283"}}' --testnet -o json | jq
    
{
  "data": {
    "id": "63069195-bc51-41bd-80d7-0ab84b98e283",
    "amount": "10000",
    "description": "Air Jordan High Black Red"
  }
}
```

### Pay Invoice

`customer` can now pay invoice by transferring `usdx.c` coin to the smart contract.

First, make sure `customer` has sufficient `usdx.c` coin balance to pay for the invoice:

```bash
provenanced q bank balances $(provenanced keys show -a customer --home build/node0 --keyring-backend test --testnet) \
    --testnet -o json | jq

{
  "balances": [
    {
      "denom": "nhash",
      "amount": "100000000000"
    },
    {
      "denom": "usdx.c",
      "amount": "100000"
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "0"
  }
}
```

Pay invoice by transferring 10,000 quantity of `usdx.c` from customer to smart contract:

```bash
provenanced tx wasm execute tp153r9tg33had5c5s54sqzn879xww2q2egektyqnpj6nwxt8wls70qrv2qq2 \
    '{"pay_invoice":{"id":"63069195-bc51-41bd-80d7-0ab84b98e283"}}' \
    --amount 10000usdx.c \
    --from customer \
    --home build/node0 --keyring-backend test \
    --chain-id chain-local \
    --gas auto --gas-prices 1905nhash --gas-adjustment 1.3 \
    --testnet \
    --yes -o json | jq
```

The payment was sent to the recipient address. You can confirm this by querying by its address coin balance:

```bash
provenanced q bank balances $(provenanced keys show -a recipient --home build/node0 --keyring-backend test --testnet) \
    --testnet -o json | jq

{
  "balances": [
    {
      "denom": "usdx.c",
      "amount": "10000"
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "0"
  }
}
```

### Cancel

`merchant` can cancel an invoice that is left unpaid to remove it from smart contract state.

```bash
provenanced tx wasm execute tp153r9tg33had5c5s54sqzn879xww2q2egektyqnpj6nwxt8wls70qrv2qq2 \
    '{"cancel_invoice":{"id":"63069195-bc51-41bd-80d7-0ab84b98e283"}}' \
    --from merchant \
    --home build/node0 --keyring-backend test \
    --chain-id chain-local \
    --gas auto --gas-prices 1905nhash --gas-adjustment 1.3 \
    --testnet \
    --yes -o json | jq
```

If you query for the invoice, it will no longer be found:

```bash
provenanced query wasm contract-state smart tp153r9tg33had5c5s54sqzn879xww2q2egektyqnpj6nwxt8wls70qrv2qq2 \
    '{"get_invoice":{"id":"63069195-bc51-41bd-80d7-0ab84b98e283"}}' --testnet -o json | jq
    
Error: rpc error: code = InvalidArgument desc = invoice::state::Invoice not found: query wasm contract failed: invalid request
```

### Query Contract Info

```bash
provenanced query wasm contract-state smart tp153r9tg33had5c5s54sqzn879xww2q2egektyqnpj6nwxt8wls70qrv2qq2 \
    '{"get_contract_info":{}}' --testnet -o json | jq

{
  "data": {
    "admin": "tp1m4arun5y9jcwkatq2ey9wuftanm5ptzsg4ppfs",
    "recipient": "tp1cxjkp6sxregvhqfqc74ythsha6g00dnry9ef6m",
    "denom": "usdx.c",
    "business_name": "Shoe Co, LLC"
  }
}
```

### Query Version Info

```bash
provenanced query wasm contract-state smart tp153r9tg33had5c5s54sqzn879xww2q2egektyqnpj6nwxt8wls70qrv2qq2 \
    '{"get_version_info":{}}' --testnet -o json | jq
    
{
  "data": {
    "contract": "invoice",
    "version": "0.1.0"
  }
}
```
