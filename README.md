# What is this?
It is a tool to make **m**ouse**less** selection of text in
the terminal easier.

It allows you to either pipe the text into it and select it
by using displayed hints:

TODO Add Asciinema about this when the regexes are stable

or to use it inside tmux to grab the text on the screen:

TODO Add Asciinema about this when the regexes are stable

# How do I install it?
Clone the repostiory, build it with `cargo build --release`
and place the resulting `mless` executable somewhere in
`$PATH`.

To use it with tmux, see [tmux integration](./tmux/README.md).

# How is this different from tmux-fingers, tmux-picker and similar tools?

- It aims to be more configurable, allowing you to easily configure
both appearance and behavior
- It is usable without tmux, e.g. directly from command-line or from scripts

That being said, I have used and loved both of these and they serve as
a basis of how mless works.
