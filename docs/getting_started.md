## Getting Started with Freqtrade Operator

This document provides a step-by-step guide to getting started with the Freqtrade Operator, including an explanation of each section of the example Custom Resource Definition (CRD).

### Prerequisites

- Kubernetes cluster
- Helm installed
- Freqtrade Operator installed (follow the installation instructions in the main README)

### Example CRD

Below is a minimal working example of the CRD (mimicking the `examples/basic.bot.yaml` file):

```yaml
apiVersion: freqtrade.io/v1alpha1
kind: Bot
metadata:
  name: example-bot
  namespace: default
spec:
  exchange: kucoin
  config:
    max_open_trades: 4
    stake_currency: USDT
    stake_amount: 0.05
    tradable_balance_ratio: 0.99
    fiat_display_currency: USD
    timeframe: 5m
    dry_run: true
    dry_run_wallet: 10000
    cancel_open_orders_on_exit: false
    unfilledtimeout:
      entry: 10
      exit: 10
      exit_timeout_count: 0
      unit: minutes
    entry_pricing:
      price_side: same
      use_order_book: true
      order_book_top: 1
      price_last_balance: 0.0
      check_depth_of_market:
        enabled: false
        bids_to_ask_delta: 1
    exit_pricing:
      price_side: same
      use_order_book: true
      order_book_top: 1
    exchange:
      ccxt_config: {}
      ccxt_async_config: {}
      pair_whitelist:
        - ALGO/USDT
        - ATOM/USDT
        - ETH/USDT
      pair_blacklist:
        - BNB/.*
    pairlists:
      - method: StaticPairList
    telegram:
      enabled: false
    initial_state: running
    force_entry_enable: false
    internals:
      process_throttle_secs: 5
  api:
    enabled: true
    host: 0.0.0.0
    port: 8081
  secrets:
    api:
      username:
        value: someuser
      password:
        value: somepassword
      ws_token:
        value: sometoken
  strategy:
    name: MyStrategy
    source: |
      *Retracted for brevity, for full strategy source see `examples/basic.bot.yaml`*
```

### Explanation of the CRD

The spec consists of a few main sections:

- `config`: This is the section where your bot's `config.json` content will go in, but in YAML format (the content gets converted and injected into the bot as JSON). Most config options are available, and the operator makes no assumption on the content of this section or what version of the bot you are using. There are a few fields that are reserved by the operator, and if present, will cause validation to fail when creating the bot. These fields are:
    - `config.add_config_files`
    - `config.recursive_strategy_search`
    - `config.strategy_path`
    - `config.strategy`
    - `config.bot_name`
    - `config.db_url`
    - `config.api_server.enabled`
    - `config.api_server.listen_ip_address`
    - `config.api_server.listen_port`
    - `config.api_server.jwt_secret_key`
    - `config.api_server.username`
    - `config.api_server.password`
    - `config.api_server.ws_token`
    - `config.telegram.token`
    - `config.telegram.chat_id`
    - `config.exchange.name`
    - `config.exchange.key`
    - `config.exchange.secret`
    - `config.exchange.password`
    - `config.freqai.enabled`

-  `exchange`: This is the name of the exchange to use. It gets injected as the `config.exchange` field in the bot's config. This field is required.

- `database`: This field is the connection string for the database. It is optional, and defaults to "sqlite:///database.db".

- `api`: This section defines the API server settings for the bot instance. If `enabled` is set to `true`, the API server will be enabled for the bot instance. The `host` and `port` fields define the IP address and port number that the API server will listen on. If not specified, the API server will listen on all IP addresses (`0.0.0.0`) and port `8081`. A Service will be created if the API server is enabled. Some control on what service type and additional ports can be specified in the `spec.service` field.

- `secrets`: This section defines the secrets that the bot instance will use. The `api` section defines the secrets that will be used for the API server. The `exchange` section defines the secrets that will be used for the exchange. The `telegram` section defines the telegram token and chat ID, both are optional. The `api` section is optional, but if present, the `username` and `password` fields are required. The `exchange` section is required. The `key`, `secret`, and `password` fields are optional. The `ws_token` field is optional, but if present, it will be used for the API server's websocket endpoint.

- `strategy`: This section defines the strategy that the bot instance will use. The `name` field defines the class name for the strategy (this is what Freqtrade uses to discover the strategy). The `source` field defines the actual source code of the strategy. The `configMapName` field defines the name of the ConfigMap that contains the `strategy.py` key with the strategy class source code. The `name` is required, and the `source` and `configMapName` fields are optional.

- `model`: This section defines the freqai model information that the bot instance will use. If this section exists then it assumes freqai is enabled. The `name` field is required and defines the name of the model class. The `source` field defines the actual source code of the model class as a string, and the `configMapName` field defines the name of the ConfigMap that contains the `model.py` key with the model class source code. Both the `source` and `configMapName` fields are optional.

For information about all possible fields, please see the [reference](reference.md).