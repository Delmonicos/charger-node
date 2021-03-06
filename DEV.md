# Development setup for charger offchain worker

By default, **Alice** account is automatically configured as the 'Chargers' organization owner, and **Bob** account is configured as a charger, the key is loaded into the keystore and addedd to the 'Chargers' organization.

The following steps allow you to configure the charger account manually.

## Accounts

### Generate charger account

Use [subkey](https://substrate.dev/docs/en/knowledgebase/integrate/subkey) to generate a new keypair:

```
subkey generate
```
### Charger organization

By default, Alice account is configurated as Organization Owner for chargers.

## Start the substrate node

To start with debug logs:

```
cargo run -- -lpallet_charge_session=debug,charger_service=debug --dev
```

## Transfer units to charger account

With the substrate node running, make a transfer of 1 unit to Account ID of the charger (output of the `subkey generate` command).

## Add the charger account to the chargeur organization

Using Alice account, add the Account Id of the charger in the Alice's organization

## Register the charger account in the keystore

```
curl -X POST 'localhost:9933' \
  --header 'Content-Type: application/json' \
  --data-raw '{
      "jsonrpc":"2.0",
      "id":1,
      "method":"author_insertKey",
      "params": [
        "chrg",
        "{{ Secret seed generated by subkey command }}",
        "{{ Public key generated by subkey command, format 0x..... }}"
      ]
  }'
```