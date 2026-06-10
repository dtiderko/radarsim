#! /usr/bin/env bash

cargo build --profile=wasm-release --target wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.123
wasm-bindgen --no-typescript --target web \
    --out-dir ./out/ \
    --out-name "radarsim" \
    ./target/wasm32-unknown-unknown/wasm-release/radarsim.wasm
wasm-opt -Oz -o ./out/radarsim_bg.wasm ./out/radarsim_bg.wasm

cp -r ./assets/ ./out/assets/

cat >./out/index.html <<EOL
<!doctype html>
<html lang="en">

<head>
  <meta charset="utf-8">
  <title>RadarSim</title>
  <style>
    html,
    body,
    canvas {
      height: 100% !important;
      width: 100% !important;
    }
  </style>
</head>

<body style="margin: 0px;">
  <script type="module">
    import init from './radarsim.js'

    init().catch((error) => {
      if (!error.message.startsWith("Using exceptions for control flow, don't mind me. This isn't actually an error!")) {
        throw error;
      }
    });
  </script>
</body>

</html>
EOL
