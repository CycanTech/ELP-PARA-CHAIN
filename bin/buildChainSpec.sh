../polkadot/target/release/polkadot build-spec --disable-default-bootnode --chain rococo-local --raw > rococo-local-raw.json
../polkadot/target/release/polkadot build-spec --disable-default-bootnode --chain rococo-local > rococo-local-src.json

../phoenix/target/release/phoenix-collator build-spec --disable-default-bootnode > phoenix-src.json
../phoenix/target/release/phoenix-collator build-spec --chain phoenix-src.json --disable-default-bootnode --raw > phoenix-raw.json

