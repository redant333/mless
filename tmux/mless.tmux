#!/usr/bin/env bash
set -e

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
            echo "MLESS_PATH is set to $MLESS_PATH which is not an executable file" >&2
            exit 1
        fi
    elif [ -x "$(command -v "mless")" ]; then # Priority two - explicitly set path
        echo "mless"
        return
    else
        echo "mless is not available. Make it available in PATH or set environment variable MLESS_PATH" >&2
        exit 1
    fi
}

###############################################################################
# Determine a command that can be used in the following way:
#   echo text | <command>
# to place the word "text" to clipboard.
#
# Outputs:
#   The command on stdout, if it could be determined, the error on stderror
#   otherwise.
# Returns:
#   0 if the command could be determined, 1 otherwise
###############################################################################
function get_copy_command() {
    if [ -x "$(command -v xclip)" ]; then # X11
        echo "xclip -selection clipboard"
    elif [ -x "$(command -v clip.exe)" ]; then # WSL
        echo "clip.exe"
    else
        echo "Could not detect copy command, set environment variable MLESS_COPY_PIPE_COMMAND" >&2
        exit 1
    fi
}

mless_copy_hotkey=${MLESS_COPY_HOTKEY-"M-z"}
mless_select_and_paste_hotkey=${MLESS_SELECT_AND_PATE_HOTKEY-"M-q"}

mless_copy_pipe_command=$(get_copy_command)
mless_executable=$(get_mless_executable)

current_dir=$(dirname "$(realpath "$0")")

tmux bind -n "$mless_copy_hotkey" \
    run-shell "'$current_dir/mless_wrapper.sh' '$mless_executable' '$mless_copy_pipe_command'"

tmux bind -n "$mless_select_and_paste_hotkey" \
    run-shell "'$current_dir/mless_wrapper.sh' '$mless_executable'"
