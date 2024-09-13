#!/usr/bin/env bash
#
# Execute mless to select text from the current tmux pane.
# This script does roughly the following:
#  - Capture the contents of the currently active pane into a file
#  - Prepare a side window with a pane with the same dimensions as the
#    currently active one
#  - Execute mless in the side window
#  - Swap the current pane and its equivalent in the side pane to make
#    mless visible to the user
#  - Wait for the user to finish selection
#  - Paste the selected text (if any) in the original pane
#  - Clean up and make all the windows as they were before the execution
#    of this script
#
# Heavily based on the approach taken by tmux-picker
# https://github.com/pawel-wiejacha/tmux-picker/tree/827845f89044fbfb3cd73905f000340bbbda663a
set -ue

readonly SIDE_WINDOW_NAME="[mless]"
readonly CAPTURE_FILE="/tmp/mless_captured"

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

###############################################################################
# Execute mless, paste the selected text and clean up the side window.
# NOTE: This is not executed in the main process as the rest of the script!
# It is intended to be executed independently in a pane.
#
# Arguments:
#   ID of the original pane from which the selection is performed.
#   ID of the pane in which this function and mless are executing.
#   ID of the side window.
#   ID of the file that contains the capture of the original pane.
###############################################################################
function execute_mless() {
    local -r BUFFER_NAME="mless-buff"

    local selection_source_pane_id=$1
    local picker_pane_id=$2
    local side_window_id=$3
    local select_from_file=$4

    cmd="mouseless-selector $select_from_file"

    tmux swap-pane -s "$selection_source_pane_id" -t "$picker_pane_id"

    selected_text=$($cmd)

    if [[ "$selected_text" != "" ]]; then
        echo -n "$selected_text" | tmux loadb -b "$BUFFER_NAME" - && tmux paste-buffer -b "$BUFFER_NAME" -t "$selection_source_pane_id"
    fi

    tmux swap-pane -s "$picker_pane_id" -t "$selection_source_pane_id"
    tmux kill-window -t "$side_window_id"
}

# Capture the current pane
selection_source_pane_id=$(tmux list-panes -F "#{pane_id}:#{?pane_active,active,nope}" | grep active | cut -d: -f1)
capture_pane "$selection_source_pane_id" "$CAPTURE_FILE"

# Initialize the side window and get the IDs
picker_ids=$(init_side_window "$SIDE_WINDOW_NAME")
picker_pane_id=$(cut -f1 -d: <<< "$picker_ids")
side_window_id=$(cut -f2 -d: <<< "$picker_ids")

# Prepare the command that will be executed to run mless in the prepared pane
# The tr and sed parts are mean to transform the function into a one liner
execute_mless_contents=$(declare -f execute_mless | tr '\n' ' ' | sed 's/\}/; }/')
args="\"$selection_source_pane_id\" \"$picker_pane_id\" \"$side_window_id\" \"$CAPTURE_FILE\""

tmux respawn-pane -k -t "$picker_pane_id" bash -c "$execute_mless_contents ; execute_mless $args"
