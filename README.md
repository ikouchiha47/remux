# remux

tmux native terminal emulator.

This is mainly possible because of the work done by the iTerm2 guy, building the `Control Mode` in termux.
If not for that, I would have to build a `libtmux`, to interract with `tmux`.

## requirements

- rust 1.82.0

## libraries

- eframe, to render the terminal natively
- tokio, because we will be waiting on tmux to send output
- vte, to parse ascii codes to render in terminal
- termwiz, a rust library by wex used for wezterm to parse anscii stuff

## resources:

- [https://poor.dev/blog/terminal-anatomy/](https://poor.dev/blog/terminal-anatomy/)
- [https://github.com/tmux/tmux/wiki/Control-Mode](https://github.com/tmux/tmux/wiki/Control-Mode)

## workings

1. The Usual Terminal–Shell–PTY Setup

- Normally, a terminal emulator (like GNOME Terminal, xterm, iTerm2, etc.) creates a pty and spawns a shell on it.
- When you type characters into your terminal, the terminal sends them over the pty’s STDIN channel to the shell.
- The shell’s output comes back on the pty’s STDOUT channel, which the terminal emulator renders onscreen.

This means if I were to write my own terminal emulator from scratch, I’d manually create the pty, spawn the shell on it, and handle input/output.

2. Tmux’s Role

Tmux internally does the same sort of pty management but for multiple shells/panes at once. Specifically:

-For each pane created in tmux, tmux spins up a pty (and usually spawns a shell on it, unless you run another program).
-Tmux reads the shell’s output from that pty.
-Tmux composes the text from multiple shells/panes into a single layout and outputs that to your real terminal—or sends structured messages in `Control Mode`.

So, talk to tmux (via tmux -CC or by sending it commands). Parse the commands/events sent by tmux and draw the ui.
