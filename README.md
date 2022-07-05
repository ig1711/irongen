# irongen

Searches `XDG_DATA_DIRS/aaplications` and `XDG_DATA_HOME/applications` and lets user pick one using `fzf`

## Installation

Install rust and cargo then if you want to install `irongen` in your `root`'s `bin` directory (requires root or sudo), use the command

```sh
cargo install --root --git https://github.com/ig1711/irongen.git
```

Or if you want to install it in `$CAGRO_HOME` (by default `$HOME/.cargo`), use

```sh
cargo install --git https://github.com/ig1711/irongen.git
```

## Usage

This program depends on `fzf`
<br>
Make sure you have it installed and it is included in `$PATH`

#### With Hyprland window manager

- Create a bash script like this `$HOME/runfzf.sh`

```sh
#!/bin/bash
exec hyprctl dispatch exec $(irongen)
```

- Edit the `hyprland.conf` to include these

```sh
# add a window rule to open the terminal in floating mode
windowrule=float,floating-term

# add a keybind to call the script created previously, using a terminal. I'm using foot term here
# the -a flag for foot term allows to set an `app-id`
# the -w flag is for window size
bind=SUPER,D,exec,foot -w 1366x768 -a floating-term $HOME/runfzf.sh
```