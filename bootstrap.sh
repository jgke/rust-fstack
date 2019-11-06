#!/bin/sh

trap "exit" INT

set -eu

cd "$(dirname "$0")"

tmux new-session \; \
    send-keys "cargo watch -s \"clear && cargo check && cargo test && cargo run\"" C-m \; \
    split-window -h \; \
    send-keys "(cd src/frontend; CARGO_TARGET_DIR=../../target cargo web start)" C-m \; \
    select-pane -t 0 \; \
    split-window -v \; \
    send-keys "(cd src/frontend; cargo watch -s \"clear && sass-rs < scss/site.scss > static/site.css && cargo web test --nodejs\")" C-m \; \
    select-pane -t 2 \; \
    split-window -v \; \
    send-keys "(cd db; ./start-db.sh)" C-m \;
