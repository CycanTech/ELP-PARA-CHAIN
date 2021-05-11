# Alice
../polkadot/target/release/polkadot \
  --base-path ../data/rococo-relay1 \
  --chain rococo-local-raw.json \
  --rpc-methods Unsafe \
  --ws-port 9944 \
  --validator \
  --alice \
  --port 50556 \
  --ws-external \
  --rpc-external \
  --rpc-cors all \
>>../log/relayA.out 2>&1 &

sleep 1

# Bob 
../polkadot/target/release/polkadot \
  --base-path ../data/rococo-relay2 \
  --chain rococo-local-raw.json \
  --ws-port 9943 \
  --validator \
   --bob \
  --port 50555 \
>../log/relayB.out 2>&1 &

