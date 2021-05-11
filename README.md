```bash
git clone https://github.com/CycanTech/ELP-PARA-CHAIN.git

cd phoenix
cargo build --release

cd ../polkadot
cargo build --release

cd ../bin

# Export genesis state
./buildChainGene.sh

# Export spec file
./buildChainSpec.sh

### Launch the Rococo RelayChain
./startRelayChain.sh

### Launch the Phoenix Parachain
./startParaChain.sh


### Register the phoenix parachain
# polkadot.js UI 
https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9944

UI menu level
sudo
  parasSudoWrapper
    sudoScheduleParaInitialize(id, genesis)

input items:
            id <- 6806 
   genesisHead <- genesis-state(above)
validationCode <- genesis-wasm (above)
     parachain <- true


### view log
tail -f ../log/Collator1.out 

### Launch the Phoenix Parachain
./stopChain.sh


```
