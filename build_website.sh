#! /usr/bin/env bash

trunk build --cargo-profile=wasm-release
cp -r ./assets/ ./dist/assets/

cd ./dist || exit
for file in *.wasm; do
  wasm-opt -Oz -o "$file" "$file"
done

