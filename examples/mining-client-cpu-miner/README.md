# Mining Client CPU Miner Example

This example demonstrates some basic patterns for using `tower-stratum` to build a Sv2 Client Service, with focus on the Mining Protocol.

More specifically, we implement a CPU miner.

The user chooses between two possible operation modes:
- Standard Channel (no extranonce rolling aka Header Only mining)
- Extended Channel (extranonce rolling)

## `config` module

The `config` module is reponsible for pasing a `.toml` file, which should contain the basic information needed for the Client Service:

- `server_addr`: IP and port of the Sv2 Mining Server
- `auth_pk`: authority Public Key of the Sv2 Mining Server
- `extranonce_rolling`: chooses between Standard vs Extended Channel mode

# `client` module

The `client` module illustrates how to utilize the `Sv2ClientService` while building some applciation. Three methods are implemented:
- `new`: loads a `Sv2ClientServiceConfig`, where the most relevant info are:
  - `supported_protocols`: a reference to the Mining Protocol
  - `mining_config`: a reference to a `Sv2ClientServiceMiningConfig` setting the `extranonce_rolling` flag
  - `start`: calls `Sv2ClientService`'s `start` method
  - `shutdown`: calls `Sv2ClientService`'s `shutdown` method


## `handler` module

The `handler` module illustrates how to implement the `Sv2MiningClientHandler` trait, with functions that are responsible for handling different messages that a Client could potentially receive under the Sv2 Mining Protocol.