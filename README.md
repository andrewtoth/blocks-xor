# Bitcoin Core blocks directory XOR obfuscator

Bitcoin Core v28 and later versions have the ability to obfuscate the blocks directory while it is stored on disk.
However, this only takes effect if the blocks directory is fresh, i.e. you will have to resync the node from scratch.
This simple program will create a random obfuscation key and XOR all block files. It currently only works for mainnet,
and only if your blocks are stored in the default data directory.

## Running

Shut down bitcoind and run `cargo run --release`.
This can take some time, so please be patient.

The program can be safely stopped and restarted until all files
have been obfuscated.
