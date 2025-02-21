#!/usr/bin/env bash
set -e

###############################################################################
# Display the given message on the next tmux session start
#
# Arguments:
#   The message to be displayed
###############################################################################
function display_message() {
    tmux set-hook -g session-created "display-message -d 0 '$1' ; set-hook -ug session-created"
}

###############################################################################
# Determine the command that can be used to run mless. This uses either the
# value of MLESS_PATH variable or searches PATH.
#
# Globals:
#   MLESS_PATH
# Outputs:
#   The command to run mless, if it could be determined, the error on stderror
#   otherwise.
# Returns:
#   0 if the command could be determined, 1 otherwise.
###############################################################################
function get_mless_executable() {
    if [ "$MLESS_PATH" != "" ] ; then # Priority one - explicitly set path
        if [ -x "$MLESS_PATH" ]; then
            echo "$MLESS_PATH"
            return
        else
            display_message "MLESS_PATH is set to $MLESS_PATH which is not an executable file"
            exit 1
        fi
    elif [ -x "$(command -v "mless")" ]; then # Priority two - mless in PATH
        echo "mless"
        return
    else
        display_message "mless is not available. Make it available in PATH or set environment variable MLESS_PATH"
        exit 1
    fi
}

###############################################################################
# Determine a command that can be used in the following way:
#   echo text | <command>
# to place the word "text" to clipboard.
#
# Globals:
#   MLESS_COPY_PIPE_COMMAND
# Outputs:
#   The command on stdout, if it could be determined, the error on stderror
#   otherwise.
# Returns:
#   0 if the command could be determined, 1 otherwise
###############################################################################
function get_copy_command() {
    if [ "$MLESS_COPY_PIPE_COMMAND" != "" ]; then # Explicitly set
        echo "$MLESS_COPY_PIPE_COMMAND"
    elif [ -x "$(command -v xclip)" ]; then # X11
        echo "xclip -selection clipboard"
    elif [ -x "$(command -v clip.exe)" ]; then # WSL
        echo "clip.exe"
    else
        display_message "Could not detect copy command, set environment variable MLESS_COPY_PIPE_COMMAND"
        exit 1
    fi
}

copy_hotkey=${MLESS_BIND_COPY_MODE-"M-c"}
select_and_paste_hotkey=${MLESS_BIND_SELECT_AND_PASTE_MODE-"M-f"}

copy_pipe_command=$(get_copy_command)
mless_executable=$(get_mless_executable)

current_dir=$(dirname "$(realpath "$0")")

tmux bind -n "$copy_hotkey" \
    run-shell "'$current_dir/mless_wrapper.sh' '$mless_executable' '$copy_pipe_command'"

tmux bind -n "$select_and_paste_hotkey" \
    run-shell "'$current_dir/mless_wrapper.sh' '$mless_executable'"
