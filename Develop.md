# findings

Running tmux in client mode and checking the protocol.

## running a new session

```shell
[1]> tmux -Ltest -C new

%begin 1734846703 366 0
%end 1734846703 366 0
%window-add @1
%sessions-changed
%session-changed $1 1
%output %1 Welcome to fish, the friendly interactive shell\015\012Type \033[32mhelp\033[m\017 for instructions on how to use fish\015\012
%output %1 \033[?2004h
%output %1 \033]0;~/d/remux\007\033[30m\033[m\017\015
%output %1 \033[92malexday\033[m\017@\033[m\017Alexs-MacBook-Air\033[m\017 \033[32m~/d/remux\033[m\017 (master)\033[m\017> \033[K\015\033[46C

```

Pressing `<Enter>` on an empty line, exits the tmux session.

> Every command produces one block of output. This is wrapped in two guard lines: either %begin and %end if the command succeeded, or %begin and %error if it failed.

Every %begin, %end or %error has three arguments:

- the time as seconds from epoch;
- a unique command number;
- flags, at the moment this is always one.

## list-session

```shell
list-sessions

%begin 1734846843 373 1
0: 1 windows (created Sun Dec 22 11:21:16 2024)
1: 1 windows (created Sun Dec 22 11:21:43 2024) (attached)
%end 1734846843 373 1
```

shortcut, `ls`

```shell
ls -F '#{session_id} "#{q:session_name}"'

%begin 1734847244 380 1
$0 "0"
$1 "1"
%end 1734847244 380 1
```

## send commands

```shell
send ls './' Enter

%begin 1734847012 377 1
%end 1734847012 377 1
%output %1 ls./\015\033[56C\015\012
%output %1 \033[30m\033[m\017
%output %1 \033[?2004l
%output %1 \033[?1004l
%output %1 \033]0;ls./ ~/d/remux\007\033[30m\033[m\017\015
%output %1 fish: Unknown command: ls./\015\012
%output %1 \033[?1004h
%output %1 \033[2m⏎\033[m\017                       \015⏎ \015\033[K\033[?2004h
%output %1 \033]0;~/d/remux\007\033[30m\033[m\017\015
%output %1 \033[92malexday\033[m\017@\033[m\017Alexs-MacBook-Air\033[m\017 \033[32m~/d/remux\033[m\017 (master)\033[m\017 \033[m\017\033[31m[\033[1m\033[31m127\033[m\017\033[31m]\033[m\017\033[m\017> \033[K\015\033[52C
```

This looks how a default fish shell looks like.

- So I need to parse according to spec.
- Render ASCII characters on eframe:egui
