## Compiling

To compile all the contracts, run the following in the repo root:

```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.4
```

This will compile all packages in the `contracts` directory and output the
stripped and optimized wasm code under the `artifacts` directory as output,
along with a `checksums.txt` file.

If you hit any issues there and want to debug, you can try to run the
following in each contract dir:
`RUSTFLAGS="-C link-arg=-s" cargo build --release --target=wasm32-unknown-unknown --locked`

## Creating a new contract

You can start with [cosmwasm-template](https://github.com/CosmWasm/cosmwasm-template) as a basis:

```bash
cd contracts
cargo generate --git https://github.com/CosmWasm/cosmwasm-template.git --name CONTRACT_NAME
cd CONTRACT_NAME

# remove unneeded files
rm -rf .circleci .github .git
rm .cargo-ok .editorconfig .gitignore rustfmt.toml
rm Developing.md Importing.md Publishing.md LICENSE NOTICE

# regenerate schema for CI tests
cargo schema

git add .
```

Then add it to CI, by editing `.circleci/config.yml`. 

Finally, update `Cargo.toml` to use the current version used by all other contracts in this repo.
