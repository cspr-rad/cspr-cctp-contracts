# Cross-Chain Transfer Protocol (CCTP) on Casper

# Chain-agnostic Accounts
Casper uses an `Address` type that wraps `Contract` and `User` addresses. In order to make this model compatible with CCTP we decided to drop the first Byte of each `Address` in our Stablecoin accounting and represent Casper Accounts in a chain-agnostic way. For this purpose functions `generic_address(SOME_CASPER_ACCOUNT) -> [u8;32]` and `generic_address_to_*(SOME_GENERIC_ADDRESS) -> Address` were introduced. 

*See `src/lib.rs`*

# MessageTransmitter Smart Contract

`init`

| Parameter | Type | Description |
|-----------|------|-------------|
| local_domain | u32 | The identifier of the local chain |
| version | u32 | The version of the MessageTransmitter Contract |
| max_message_body_size | u256 | The maximum size of a CCTP Message body |
| next_available_nonce | u64 | The starting nonce of this MessageTransmitter |
| signature_threshold | u32 | The initial attestation threshold for this MessageTransmitter |
| owner | Address | Casper Address of the MessageTransmitter owner |


`send_message`

| Parameter | Type | Description |
|-----------|------|-------------|
| destination_domain | u32 | The identifier of the remote chain |
| recipient | [u8;32] | Chain-agnostic remote address of the recipient TokenMessengerMinter |
| message_body | Bytes | Casper-wrapped Bytes of the message body |

`send_message_with_caller`

| Parameter | Type | Description |
|-----------|------|-------------|
| destination_domain | u32 | The identifier of the remote chain |
| recipient | [u8;32] | Chain-agnostic remote address of the recipient MessageTransmitter |
| message_body | Bytes | Casper-wrapped Bytes of the message body |
| destination_caller | [u8;32] | Specific instance of MessageTransmitter that is allowed to process this message |

`replace_message`

| Parameter | Type | Description |
|-----------|------|-------------|
| original_message | Bytes | Casper-wrapped Bytes of the original message |
| original_attestaion | Bytes | Casper-wrapped Bytes of the attestation to the original message |
| new_message_body | Bytes | Casper-wrapped Bytes of the formatted body of the new message |
| new_destination_caller | [u8;32] | Specific instance of MessageTransmitter that is allowed to process the new message |

`receive_message`

| Parameter | Type | Description |
|-----------|------|-------------|
| data | Bytes | Casper-wrapped Bytes of the message data |
| attestation | Bytes | Casper-wrapped Bytes of the attestation |


`set_max_message_body_size`

| Parameter | Type | Description |
|-----------|------|-------------|
| new_max_message_body_size | U256 | The value that the max_message_size should be updated to |

`set_signature_threshold`

| Parameter | Type | Description |
|-----------|------|-------------|
| new_signature_threshold | u32 | The value that the signature_threshold should be updated to |

`transfer_ownership`

| Parameter | Type | Description |
|-----------|------|-------------|
| new_pending_owner | Address | Casper Address of the new owner account |

`accept_ownership`

| Parameter | Type | Description |
|-----------|------|-------------|
*This Ep does not take any arguments*

`pause`

| Parameter | Type | Description |
|-----------|------|-------------|
*This Ep does not take any arguments*

`unpause`

| Parameter | Type | Description |
|-----------|------|-------------|
*This Ep does not take any arguments*

`is_used_nonce`

| Parameter | Type | Description |
|-----------|------|-------------|
| nonce | u64 | The value of the nonce to be checked |
| account | [u8;32] | The Casper Account, formatted as a Chain-agnostic account, for which the nonce is checked |

`enable_attester`

| Parameter | Type | Description |
|-----------|------|-------------|
| new_attester | [u8;32] | The Chain-agnostic Address of the new Attester (=some PublicKey) |

`disable_attester`

| Parameter | Type | Description |
|-----------|------|-------------|
| attester | [u8;32] | The Chain-agnostic Address of the new Attester (=some PublicKey) |


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
