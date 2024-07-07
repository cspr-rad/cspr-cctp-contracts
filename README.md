# Cross-Chain Transfer Protocol (CCTP) on Casper

# MessageTransmitter Smart Contract

`init`

| Parameter | Type | Description |
|-----------|------|-------------|

`send_message`

| Parameter | Type | Description |
|-----------|------|-------------|

`send_message_with_caller`

| Parameter | Type | Description |
|-----------|------|-------------|

`replace_message`

| Parameter | Type | Description |
|-----------|------|-------------|

`receive_message`

| Parameter | Type | Description |
|-----------|------|-------------|

`set_max_message_body_size`

| Parameter | Type | Description |
|-----------|------|-------------|

`set_signature_threshold`

| Parameter | Type | Description |
|-----------|------|-------------|

`transfer_ownership`

| Parameter | Type | Description |
|-----------|------|-------------|

`accept_ownership`

| Parameter | Type | Description |
|-----------|------|-------------|

`pause`

| Parameter | Type | Description |
|-----------|------|-------------|

`unpause`

| Parameter | Type | Description |
|-----------|------|-------------|

`is_used_nonce`

| Parameter | Type | Description |
|-----------|------|-------------|

`enable_attester`

| Parameter | Type | Description |
|-----------|------|-------------|

`disable_attester`

| Parameter | Type | Description |
|-----------|------|-------------|


# TokenMessengerMinter Smart Contract

`init`

| Parameter | Type | Description |
|-----------|------|-------------|

`deposit_for_burn`

| Parameter | Type | Description |
|-----------|------|-------------|

`deposit_for_burn_with_caller`

| Parameter | Type | Description |
|-----------|------|-------------|

`replace_deposit_for_burn`

| Parameter | Type | Description |
|-----------|------|-------------|

`handle_receive_message`

| Parameter | Type | Description |
|-----------|------|-------------|

`transfer_ownership`

| Parameter | Type | Description |
|-----------|------|-------------|

`accept_ownership`

| Parameter | Type | Description |
|-----------|------|-------------|

`add_remote_token_messenger`

| Parameter | Type | Description |
|-----------|------|-------------|

`remove_remote_token_messenger`

| Parameter | Type | Description |
|-----------|------|-------------|

`link_token_pair`

| Parameter | Type | Description |
|-----------|------|-------------|

`unlink_token_pair`

| Parameter | Type | Description |
|-----------|------|-------------|

`pause`

| Parameter | Type | Description |
|-----------|------|-------------|

`unpause`

| Parameter | Type | Description |
|-----------|------|-------------|

`set_max_burn_amount_per_message`

| Parameter | Type | Description |
|-----------|------|-------------|


## Usage
It's recommended to install 
[cargo-odra](https://github.com/odradev/cargo-odra) first.

### Build
```
$ cargo odra build
```
To build a wasm file, you need to pass the -b parameter. 
The result files will be placed in `${project-root}/wasm` directory.

```
$ cargo odra build -b casper
```

### Test
To run test on your local machine, you can basically execute the command:

```
$ cargo odra test
```

To test actual wasm files against a backend, 
you need to specify the backend passing -b argument to `cargo-odra`.

```
$ cargo odra test -b casper
```
