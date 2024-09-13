#!/usr/bin/env bash
CURRENT_DIR=$(dirname $(realpath $0))

tmux bind -n "M-q" run-shell "PATH=$CURRENT_DIR/../target/debug/:$PATH $CURRENT_DIR/mless_wrapper.sh"
