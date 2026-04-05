#!/bin/bash
set -e

echo "=== Building WASM ==="
RUSTFLAGS='-C target-feature=+atomics,+bulk-memory,+mutable-globals,+simd128 -C link-arg=--shared-memory -C link-arg=--import-memory -C link-arg=--max-memory=4294967296 -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base' \
  cargo +nightly build --lib --target wasm32-unknown-unknown --release \
  --no-default-features --features wasm \
  -Z build-std=panic_abort,std

wasm-bindgen --target web --out-dir web/pkg \
  target/wasm32-unknown-unknown/release/oxide.wasm

wasm-opt -O3 \
  --enable-simd \
  --enable-threads \
  --enable-bulk-memory \
  --enable-bulk-memory-opt \
  --enable-nontrapping-float-to-int \
  --strip-debug \
  --strip-producers \
  -o web/pkg/oxide_bg.wasm \
  web/pkg/oxide_bg.wasm


cat > web/pkg/index.js << 'EOF'
export * from './oxide.js';
export { default } from './oxide.js';
EOF

sed -i '' "s|import('../../..')|import('../../../oxide.js')|g" \
       web/pkg/snippets/wasm-bindgen-rayon-*/src/workerHelpers.js

echo "=== Build complete ==="
