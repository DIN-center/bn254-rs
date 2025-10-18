# EOA to BLS Key Mapping Reference

This document provides a reference for the mapping between Ethereum addresses (EOAs), BLS keys, and operator emails.

## Key Mappings

| Key ID | Wallet Address | Operator Email |
|--------|---------------|----------------|
| key_0  | 0xD4d6208Ad752B379389405BD0a0fF42C3101D477 | *(unknown)* |
| key_1  | 0x24473514f5dE5BFaC39C5187A5A0603B28fb3207 | *(unknown)* |
| key_2  | 0x50Ad6484833bb70569609156DBD7041584DE8083 | *(unknown)* |
| key_3  | 0xB364F96672D035a53c1E0A85A93F4605aeB62c2b | *(unknown)* |
| key_4  | 0x35228f85E5e91c1bE006B9a5d76113E024BCD3eD | *(unknown)* |
| key_5  | 0x6c1D9112baBbF90B6F356C1ebE753aeb2df6FfD1 | *(unknown)* |
| key_6  | 0x7aBF46564cfd4d67E36DC8fB5DeF6a1162EBaF6b | *(unknown)* |
| key_7  | 0x093Ddc1DD7784029b32d44F0C11B29AEb8232cc9 | *(unknown)* |
| key_8  | 0x6BeA16caD793AD7F162A19cB025b0248E895a708 | *(unknown)* |
| key_9  | 0x77DE19AbDc4ba4e23256055f72c0699fA485a867 | *(unknown)* |
| key_10 | 0xC2C0dBF0E1Dd21bD8F22241932d610aF28890933 | *(unknown)* |
| key_11 | 0xC4d3D523b0e9e74cf5E97314f8336f1153749596 | *(unknown)* |
| key_12 | 0xa4F8062dd92CbF8E597B813A42FA9139236AF457 | *(unknown)* |
| key_13 | 0x9cFe65C759851B78F74d4c7B49E6772afa7eb0c0 | *(unknown)* |
| key_14 | 0xB1805090F855AeEa34C33314D918b8eA61Fa5d86 | *(unknown)* |
| key_15 | 0xa797954ffa8853cc8d718fb4CC3f9289757eE58F | *(unknown)* |
| key_16 | 0xb5A51A65B4247b87a5DDC3Bcc197b3aD7Ff82e96 | *(unknown)* |
| key_17 | 0x38833D1710175B7b6fDE3eee549fcf8F7730C696 | *(unknown)* |
| key_18 | 0xc8Fa0fC6e8EaF77BdEb3cb79957BB23A40FaE636 | *(unknown)* |
| key_19 | 0x30e1878442134188847269f4b1EFeCf70358D45A | *(unknown)* |
| key_20 | 0x87A7E059EF321Ad691069E231F9Ae3e02643eD70 | *(unknown)* |
| key_21 | 0x8f2c5305E6257Dc59240D511EB50C3E6eD127AaC | *(unknown)* |
| key_22 | 0x2A0601EDF0B57f7Fb3FA7fA152a7D07403CB8E74 | *(unknown)* |
| key_23 | 0xF59552FC9C1Be3d7457673aef8b1F49374066Ddf | *(unknown)* |
| key_24 | 0xCcE48232A063E6994Cb000e9e391f6A380c26566 | *(unknown)* |
| key_25 | 0x6A553fD421f081eea54081987acb9e8D5c7b9Ac4 | *(unknown)* |
| key_26 | 0xDce07022a2d4784a7EF4ECC8A169b9f430688753 | natefikru@gmail.com | testnet0.3
| key_27 | 0x324454186BB728a3ea55750E0618ff1B18ce6Cf8 | austin.roberts@rivet.cloud | testnet0.3
| key_28 | 0x94eF29B1c0aEdfEEbdD605FA4Fe2353152a9f322 | fred@grove.city | testnet0.3
| key_29 | 0x77270ca927b388887117ba64faeBa0047b32fD2d | account-eigenlayer@everstake.one | testnet0.3
| key_30 | 0xAE961a84F4c70059AA706F35520978f364473f17 | luke@liquify.io | testnet0.3
| key_31 | 0x1a9a784871a7b30043d5EF2497E9AcE969aF351D | kasey.alusi@validationcloud.io | testnet0.3
| key_32 | 0xaCeC71f6D0709B8C060d4AE6717E98ec69771009 | info@0xfury.com | testnet0.3
| key_33 | 0x9682F385F5f6DD632B818f96028F45D069F11F4A | devops@simplystaking.com | testnet0.3

## Recently Added (from operator_allowlist_requests table)

The following keys (key_27 through key_33) were added based on approved operator allowlist requests:

- **key_27**: Rivet Cloud (austin.roberts@rivet.cloud)
- **key_28**: Grove City (fred@grove.city)
- **key_29**: Everstake (account-eigenlayer@everstake.one)
- **key_30**: Liquify (luke@liquify.io)
- **key_31**: Validation Cloud (kasey.alusi@validationcloud.io)
- **key_32**: 0xFury (info@0xfury.com)
- **key_33**: Simply Staking (devops@simplystaking.com)

## Notes

- Keys 0-25 were part of the original key pool and don't have email mappings in the database yet
- This reference file should be kept in sync with `eoa-keymap.json`
- Email addresses are fetched from the `users` table based on `operator_allowlist_requests`
