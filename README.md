# EnergiaPro

This repository contains the source code for the EnergiaPro SDK and CLI, which
allow users to interact with the EnergiaPro API to retrieve energy consumption
measurements and related data.

## CLI

The `energiapro` CLI provides a command-line interface for users to interact
with the EnergiaPro API.

### Installation

To install the `energiapro` CLI, use one of the following methods depending
on your operating system:

```sh
# macOS & Linux (using Homebrew)
brew install nhedger/energiapro
```

### Usage

Here's how to use the `energiapro` CLI to retrieve energy consumption
measurements and return the result in `json`, `jsonl`, `csv`, or `parquet`.

> [!NOTE] You should set your username and secret key as environment variables 
> before running the command. Alternatively, you can provide them as command-line 
> options, but using environment variables is more secure and convenient.
>
> ```sh
> export ENERGIAPRO_USERNAME="your_username"
> export ENERGIAPRO_SECRET_KEY="your_secret_key"
> ```

#### Listing available installations

This example illustrates how to retrieve a list of available installations for
the given client ID.

```shell
energiapro installations \
  --format json \
  <client-id>
```

#### Retrieving measurements

```sh
energiapro measurements \
  --from "2024-01-01" \
  --to "2024-01-31" \
  --scope "lpn-json" \
  --format jsonl \
  <client-id> <installation-id> > measurements.jsonl
```

Export typing:

- `installation_id` and `timestamp` are exported as strings.
- `index_m3`, `consumption_m3`, and `consumption_kwh` are exported as numeric values.
