# tmux integration

## Features
When integrated with `tmux` through these scripts, `mless` can be used to select text from the active pane in two modes:
1. Copy mode
    - Initiated with the hotkey from `MLESS_BIND_COPY_MODE`
    - The selected text is placed in the clipboard
2. Select and paste mode
    - Initiated with the hotkey from `MLESS_BIND_SELECT_AND_PASTE_MODE`
    - The selected text is pasted in the active `tmux` pane

See [Configuration](#configuration) section for more details on customizing the behavior.

## Installation

Place [mless.tmux](./mless.tmux) and [mless_wrapper.sh](./mless_wrapper.sh) in the same folder and add this to your `.tmux.conf`:
```
run-shell /path/to/mless.tmux
```

`mless` needs to be available in `$PATH` or pointed to with `MLESS_PATH` environment variable

## Configuration
The behavior can be configured by setting environment variables with:
```
setenv -g "<variable name>" "<variable value>"
```
before executing `mless.tmux`.

> [!NOTE]
> The default keybindings should be changed before the first release.

The full list of all the variables and their defaults:

- `MLESS_PATH`:
    - Path to `mless` executable to use
    - If not specified, `mless` executable from `$PATH` is used
- `MLESS_BIND_COPY_MODE`
    - Binding to run `mless` in copy mode. Uses the same format as `tmux`.
    - Default: `M-f`
- `MLESS_COPY_PIPE_COMMAND`
    - The command to which the selected text is piped when selected in the copy mode.
    - If not set, the first applicable command from this list:
        - If `xclip` is available: `xclip -selection clipboard`
        - If `clip.exe` is avaialbe: `clip.exe`
- `MLESS_BIND_SELECT_AND_PASTE_MODE`
    - Binding to run `mless` in select and paste mode.
    - Default: `M-c`
