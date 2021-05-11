RUST_LOG=runtime=trace ../phoenix/target/release/phoenix-collator \
  --collator \
  --base-path ../data/phoenix-c1 \
  --parachain-id 6806 \
  --chain phoenix-raw.json \
  --rpc-methods Unsafe \
  --ws-external \
  --rpc-external \
  --rpc-cors all \
  --ws-port 9966 \
  --rpc-port 9955 \
  --port 40335 \
  -- \
  --execution wasm \
  --chain rococo-local-raw.json \
  --port 30335 \
> ../log/Collator1.out 2>&1 &
