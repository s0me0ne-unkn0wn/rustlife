# Conway's Game of Life in Rust implemented in an old school way

## SUMMARY
This is a self-educational project. I just wanted to play around with Rust and to implement the Game of Life when you're starting to familiarize
yourself with a new language is a kind of tradition.

The code is definitely overcomplicated, partially on purpose, as I wanted to get familiar with generics and traits, and partially because I am not good
at the language at all yet.

## SYNOPSIS
Just `cargo run` it.

For better impression, run from xterm with a smaller font: `xterm -fn -*-fixed-medium-r-*-*-6-*-*-*-*-*-*-*`

Use arrow keys to control the viewport.

Use `D` to turn inter-move delays on or off.

Use `+` and `-` to control the duration of a move.

Use `Q` to quit.

## FEATURES
* Renders to console using [pancurses](https://github.com/ihalila/pancurses) for that old school feeling
* Infinite board (well, not really, but you can use `i128` for coordinates thanks to generics)
* Keyboard-controllable viewport

## LIMITATIONS
Viewport size is limited to `i32` by `ncurses` implementation. Let's just hope nobody will ever need more.

## LICENSE
This software is distributed under the terms and conditions of Beer-ware License revision 42, full text follows:

```
 "THE BEER-WARE LICENSE" (Revision 42):
 <s0me0ne@s0me0ne.com> wrote this file.  As long as you retain this notice you
 can do whatever you want with this stuff. If we meet some day, and you think
 this stuff is worth it, you can buy me a beer in return
```
