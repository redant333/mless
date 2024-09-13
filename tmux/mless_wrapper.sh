#!/usr/bin/env bash
# Heavily based on the approach taken by tmux-picker
# https://github.com/pawel-wiejacha/tmux-picker/tree/827845f89044fbfb3cd73905f000340bbbda663a
set -ue

function init_side_window() {
    local picker_ids=$(tmux new-window -F "#{pane_id}:#{window_id}" -P -d -n "[picker]" "/bin/sh")
    local picker_pane_id=$(echo "$picker_ids" | cut -f1 -d:)
    local picker_window_id=$(echo "$picker_ids" | cut -f2 -d:)

    local current_size=$(tmux list-panes -F "#{pane_width}:#{pane_height}:#{?pane_active,active,nope}" | grep active)
    local current_width=$(echo "$current_size" | cut -f1 -d:)
    local current_height=$(echo "$current_size" | cut -f2 -d:)

    local current_window_size=$(tmux list-windows -F "#{window_width}:#{window_height}:#{?window_active,active,nope}" | grep active)
    local current_window_width=$(echo "$current_window_size" | cut -f1 -d:)
    local current_window_height=$(echo "$current_window_size" | cut -f2 -d:)

    tmux split-window -d -t "$picker_pane_id" -h -l "$((current_window_width - current_width - 1))" '/bin/sh'
    tmux split-window -d -t "$picker_pane_id" -l "$((current_window_height - current_height - 1))" '/bin/sh'

    echo "$picker_ids"
}

function capture_pane() {
    local pane_id=$1
    local out_path=$2
    local pane_info=$(tmux list-panes -s -F "#{pane_id}:#{pane_height}:#{scroll_position}:#{?pane_in_mode,1,0}" | grep "^$pane_id")

    local pane_height=$(echo $pane_info | cut -d: -f2)
    local pane_scroll_position=$(echo $pane_info | cut -d: -f3)
    local pane_in_copy_mode=$(echo $pane_info | cut -d: -f4)

    local start_capture=""

    if [[ "$pane_in_copy_mode" == "1" ]]; then
        start_capture=$((-pane_scroll_position))
        end_capture=$((pane_height - pane_scroll_position - 1))
    else
        start_capture=0
        end_capture="-"
    fi

    tmux capture-pane -e -J -p -t $pane_id -E $end_capture -S $start_capture > $out_path
}

function exec_in_pane() {
    local pane_id=$1
    local command=$2

    tmux send-keys -t $pane_id "$command"
    tmux send-keys -t $pane_id Enter
}

function execute_mless() {
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
}

capture_file="/tmp/mless_captured"
selection_source_pane_id=$(tmux list-panes -F "#{pane_id}:#{?pane_active,active,nope}" | grep active | cut -d: -f1)

capture_pane "$selection_source_pane_id" "$capture_file"

picker_ids=`init_side_window`
picker_pane_id=$(echo "$picker_ids" | cut -f1 -d:)
side_window_id=$(echo "$picker_ids" | cut -f2 -d:)

print_stuff_text=$(declare -f execute_mless | tr '\n' ' ' | sed 's/\}/; }/')
args="\"$selection_source_pane_id\" \"$picker_pane_id\" \"$side_window_id\" \"$capture_file\""
cmd="bash -c '$print_stuff_text ; execute_mless $args'"

exec_in_pane "$picker_pane_id" "$cmd"
