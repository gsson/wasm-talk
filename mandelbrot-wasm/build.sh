mkdir -p web/wasm
cd crate
cargo build --release &&\
  wasm-opt --strip-producers --strip \
    --remove-unused-module-elements \
    target/wasm32-unknown-unknown/release/mandelbrot.wasm -o ../web/wasm/mandelbrot.wasm
