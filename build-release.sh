#!/bin/sh

BACKEND_DIR=target/release
FRONTEND_DIR=target/wasm32-unknown-unknown/release

mkdir -p release/assets
cargo build --release
(cd src/frontend && yarn \
    && sass-rs < scss/site.scss > static/site.css \
    && CARGO_TARGET_DIR=../../target cargo web build --release)
cp $BACKEND_DIR/backend release
cp $FRONTEND_DIR/frontend.js $FRONTEND_DIR/frontend.wasm $FRONTEND_DIR/index.html src/frontend/static/site.css release/assets
