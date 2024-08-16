#!/usr/bin/env bash
set -ue

selection_source_pane_id=$1
picker_pane_id=$2
side_window_id=$3
select_from_file=$4

cmd="mouseless-selector $select_from_file"

tmux swap-pane -s "$selection_source_pane_id" -t "$picker_pane_id"

selected_text=$($cmd)

if [[ "$selected_text" != "" ]]; then
    echo -n "$selected_text" | tmux loadb -b mless-buff - && tmux paste-buffer -b mless-buff -t "$selection_source_pane_id"
fi

tmux swap-pane -s "$picker_pane_id" -t "$selection_source_pane_id"
tmux kill-window -t "$side_window_id"
