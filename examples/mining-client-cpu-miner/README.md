# Mining Client CPU Miner Example

This example demonstrates some basic patterns for using `tower-stratum` to build a Sv2 Client Service, with focus on the Mining Protocol.

More specifically, we implement a CPU miner.

## `config` module

The `config` module is reponsible for parsing a `.toml` file with the following fields:
- `server_addr`: IP and port of the Sv2 Mining Server
- `auth_pk`: authority Public Key of the Sv2 Mining Server
- `n_standard_channels`: the number of Standard Channels to open with the Sv2 Mining Server
- `n_extended_channels`: the number of Extended Channels to open with the Sv2 Mining Server
- `user_identity`: string to be used on channel opening
- `device_id`: string to be used on connection

# `client` module

The `client` module illustrates how to utilize the `Sv2ClientService` while building some applciation. Three methods are implemented for `MyMiningClient`:
- `new`: constructor
- `start`: starts all the underlying tasks of `Sv2ClientService`
- `shutdown`: kills all the underlying tasks of `Sv2ClientService`

## `handler` module

The `handler` module illustrates how to implement the `Sv2MiningClientHandler` trait, with functions that are responsible for handling different messages that a Client could potentially receive under the Sv2 Mining Protocol.

## `miner` module

The `miner` module implements the actual CPU mining logic for Extended and Standard Channels.