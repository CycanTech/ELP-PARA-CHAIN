check rust version
```bash
$ rustup show
Default host: x86_64-unknown-linux-gnu
rustup home:  /home/vm/.rustup

installed toolchains
--------------------

stable-x86_64-unknown-linux-gnu
nightly-2021-01-13-x86_64-unknown-linux-gnu
nightly-2021-03-23-x86_64-unknown-linux-gnu (default)
nightly-x86_64-unknown-linux-gnu

installed targets for active toolchain
--------------------------------------

wasm32-unknown-unknown
x86_64-unknown-linux-gnu

active toolchain
----------------

nightly-2021-03-23-x86_64-unknown-linux-gnu (default)
rustc 1.53.0-nightly (5d04957a4 2021-03-22)

```

build & run parachain

```bash
git clone https://github.com/CycanTech/ELP-PARA-CHAIN.git

cd ELP-PARA-CHAIN

cd phoenix
cargo build --release

cd ../polkadot
cargo build --release

cd ../bin

# Export genesis state
./buildChainGene.sh
  will create genesis-state & genesis-wasm .

# Export spec file
./buildChainSpec.sh
 

### Launch the Rococo RelayChain
./startRelayChain.sh

### Launch the Phoenix Parachain
./startParaChain.sh

### Register the phoenix parachain
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


### view phoenix parachain
https://polkadot.js.org/apps/?rpc=ws://127.0.0.1:9966

### view log
tail -f ../log/Collator1.out 

### stop relayChain &  paraChain
./stopChain.sh

```
