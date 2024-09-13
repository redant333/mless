#!/usr/bin/env bash
# Heavily based on the approach taken by tmux-picker
# https://github.com/pawel-wiejacha/tmux-picker/tree/827845f89044fbfb3cd73905f000340bbbda663a
set -ue

readonly SIDE_WINDOW_NAME="[mless]"

###############################################################################
# Create a window in the background and create a pane in it with the same
# dimensions as the currently active pane.
#
# To achieve the current pane dimensions, additional panes will be created.
# Each of the created panes is initialized with a /bin/sh process.
#
# Arguments:
#   Name for the created window.
# Outputs:
#   IDs of the created window and pane, separated by a colon.
###############################################################################
function init_side_window() {
    local -r WINDOW_NAME="$1"
    local -r PANE_PROCESS="/bin/sh"

    local pane_window_ids pane_id
    pane_window_ids=$(tmux new-window -F "#{pane_id}:#{window_id}" -P -d -n "$WINDOW_NAME" "$PANE_PROCESS")
    pane_id=$(cut -f1 -d: <<< "$pane_window_ids")

    local current_pane_size current_pane_width current_pane_height
    current_pane_size=$(tmux list-panes -F "#{pane_width}:#{pane_height}:#{?pane_active,active,nope}" | grep active)
    current_pane_width=$(cut -f1 -d: <<< "$current_pane_size")
    current_pane_height=$(cut -f2 -d: <<< "$current_pane_size")

    local current_window_size current_window_width current_window_height
    current_window_size=$(tmux list-windows -F "#{window_width}:#{window_height}:#{?window_active,active,nope}" | grep active)
    current_window_width=$(cut -f1 -d: <<< "$current_window_size")
    current_window_height=$(cut -f2 -d: <<< "$current_window_size")

    tmux split-window -d -t "$pane_id" -h -l "$((current_window_width - current_pane_width - 1))" '/bin/sh'
    tmux split-window -d -t "$pane_id" -l "$((current_window_height - current_pane_height - 1))" '/bin/sh'

    echo "$pane_window_ids"
}

###############################################################################
# Capture the contents of the given pane and save them in a file at the given
# path. If the file already exists, its contents will be overwritten.
#
# Arguments:
#   ID of the path to capture.
#   Path to the file where the contents will be saved.
###############################################################################
function capture_pane() {
    local pane_id=$1
    local out_path=$2

    local pane_info
    pane_info=$(tmux list-panes -s -F "#{pane_id}:#{pane_height}:#{scroll_position}:#{?pane_in_mode,1,0}" | grep "^$pane_id")

    local pane_height pane_scroll_position pane_in_copy_mode
    pane_height=$(cut -d: -f2 <<< "$pane_info")
    pane_scroll_position=$(cut -d: -f3 <<< "$pane_info")
    pane_in_copy_mode=$(cut -d: -f4 <<< "$pane_info")

    local start_capture=""
    if [[ "$pane_in_copy_mode" == "1" ]]; then
        start_capture=$((-pane_scroll_position))
        end_capture=$((pane_height - pane_scroll_position - 1))
    else
        start_capture=0
        end_capture="-"
    fi

    tmux capture-pane -e -J -p -t "$pane_id" -E "$end_capture" -S "$start_capture" > "$out_path"
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

picker_ids=$(init_side_window "$SIDE_WINDOW_NAME")
echo "<$picker_ids>" > /tmp/debug
picker_pane_id=$(echo "$picker_ids" | cut -f1 -d:)
side_window_id=$(echo "$picker_ids" | cut -f2 -d:)

print_stuff_text=$(declare -f execute_mless | tr '\n' ' ' | sed 's/\}/; }/')
args="\"$selection_source_pane_id\" \"$picker_pane_id\" \"$side_window_id\" \"$capture_file\""
cmd="bash -c '$print_stuff_text ; execute_mless $args'"

tmux respawn-pane -k -t "$picker_pane_id" bash -c "$print_stuff_text ; execute_mless $args"
