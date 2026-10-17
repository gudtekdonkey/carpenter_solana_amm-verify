# carpenter_solana_amm — verified-build source

The source of the Carpenter and FLOOR Solana programs deployed on mainnet-beta. It is published so anyone can
reproduce the on-chain bytes with `solana-verify`. Comments were removed from the Rust sources; line numbers are
unchanged, and the built programs are byte-identical to the deployed ones.

| Program | ID |
|---|---|
| `carpenter_amm` | `DdeiDx4ncHo9DXGZm1GrPe7mFKv7kNU3yx95uQEV2APA` |
| `floor_hook` | `9uXLyyZginMWdUeu22kp7XV7cpV8183xfsVqUxKi5Viy` |
| `floor_launch` | `8vfdPWfaA3b9GtJoFUwc36JKDaMfVCJ4oT9HrCrQ6Tdi` |
| `floor_holders` | `2EUkJXSwCdRAuhesw2D9FXyGfbi6Kw2yZDyQUnCwc7te` |
| `floor_vesting` | `HHiwbXs3egRwtgEPEKx1PHpBU8gq5VSbhfrGrcCd4CSC` |
| `upgrade_timelock` | `BZfAEAbonruxE1FV7zNoWnSxkUrE71dLAh4qjrLsKGWA` |

Build (per program):

```sh
solana-verify build --library-name <program> --arch v3 \
  --base-image ghcr.io/gudtekdonkey/floor-solana-verify@sha256:17bc956f184ea968c176d04622f34fb115ce0cac742aa87922e8492fb1829b2b
```

The programs are upgradeable through `upgrade_timelock` (a 10-minute delay, proposed by FLOOR's deployer key) and
pausable by the deployer key. No external security audit has been completed.

SDKs: [carpenter-sdk](https://github.com/gudtekdonkey/carpenter-sdk), [floor-sdk](https://github.com/gudtekdonkey/floor-sdk).
Third-party code: `THIRD_PARTY.md`.
