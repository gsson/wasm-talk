mkdir -p web/wasm
cd crate
cargo build --release &&\
  wasm-opt --strip-producers --strip \
    --remove-unused-module-elements \
    target/wasm32-unknown-unknown/release/hello.wasm -o ../web/wasm/hello.wasm
