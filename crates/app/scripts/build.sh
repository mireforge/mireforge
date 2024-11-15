cargo build --target wasm32-unknown-unknown --bin wgpu-examples --no-default-features --features webgpu
wasm-bindgen target/wasm32-unknown-unknown/{output_dir}/wgpu-examples.wasm --target web --no-typescript --out-dir target/generated --out-name webgpu
simple-http-server target/generated -c wasm,html,js -i --coep --coop --ip 127.0.0.1
