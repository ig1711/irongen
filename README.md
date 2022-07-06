# irongen

Searches `XDG_DATA_DIRS/applications` and `XDG_DATA_HOME/applications` and lets user pick one using `fzf`

## Installation

Install rust and cargo then use the command

```sh
cargo install --git https://github.com/ig1711/irongen.git
```

This will install `irongen` in `$CAGRO_HOME/bin` (by default `$HOME/.cargo/bin`). Include this installtion directory in your `$PATH`


## Configuration

Create a directory called `irongen` in your `XDG_CONFIG_HOME` direcotory (default `$HOME/.config`)
<br>
Create a file named `config` and put your config there
<br>
Check [example config](/example_config/config) for details


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

#### With Sway window manager

- Create a bash script like this `$HOME/runfzf.sh`

```sh
#!/bin/bash
exec swaymsg -q exec $(irongen)
```

- Edit the `sway/config` file to include these

```sh
# add a window rule to open the terminal in floating mode
for_window [app_id="floating-term"] floating enable

# add a keybind to call the script created previously, using a terminal. I'm using foot term here
# the -a flag for foot term allows to set an `app-id`
# the -w flag is for window size
bindsym $mod+d exec foot -w 1366x768 -a floating-term $HOME/runfzf.sh
```
