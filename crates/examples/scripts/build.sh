cp scripts/index.* ../../target/generated/

cargo build --target wasm32-unknown-unknown --bin swamp-app-examples --no-default-features && wasm-bindgen ../../target/wasm32-unknown-unknown/debug/swamp-app-examples.wasm --target web --no-typescript --out-dir ../../target/generated --out-name swamp-app-examples && simple-http-server ../../target/generated -c wasm,html,js -i --coep --coop --ip 127.0.0.1
