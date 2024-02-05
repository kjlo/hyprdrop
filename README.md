# Hyprdrop

Rust implementation of [hdrop](https://github.com/Schweber/hdrop).

https://github.com/kjlo/hyprdrop/assets/79338048/cdf9fff6-690c-46cb-a7a6-0b2db14ab753


## Requirements

* [Hyprland](https://github.com/hyprwm/Hyprland)
* Rust >= 1.75


## Installation
Clone the repository and build:
```sh
git clone https://github.com/kjlo/hyprdrop
cd hyprdrop
cargo install --path .
```
This will create a binary in your `$HOME/.cargo/bin`. You must verify that this address it's in your `$PATH`.


## Usage
The preferred way to use it's by adding this as bindings to your Hyprland config:
```
bind = $mainMod, U, exec, hyprdrop alacritty -i alacritty_hyprdrop
```
Additionally, if you want to launch a TUI application:
```
bind = $mainMod, I, exec, hyprdrop alacritty --identifier=bottom_hyprdrop --args=btm,-b
```
You can check the `hyprdrop --help` command to see all the available options.


>[!NOTE]
>
> The argument identifier must be a unique name if you want to use as a separate application with
> special window rules.

>[!NOTE]
>
> Check that for TUI applications it's not required to type the `-e` flag that most
> terminal emulators use when executing a command, this is implemented by Hyprdrop.

>[!WARNING]
>
> Hyprdrop was initially designed with TUI applications in mind. Theoretically, it should work with
> any GUI application or TUI application not supported (you are obligated to use the original
> class/title to identify the window). However, one consideration is that GUI apps are not usable
> with the `args` flag, as it is specifically designed for terminal emulators.

### Supported Terminal Emulators
The following list shows which terminal emulators are supported by Hyprdrop:
| Terminal | Supported | Window Identifier (for Hyprland Config)|
|--------------- | ----- | -------- |
| Alacritty      | yes   | class    |
| Kitty          | yes   | class    |
| Wezterm        | yes   | class    |
| Gnome Terminal | yes   | title    |
| Foot           | yes   | title    |
| Konsole        | yes[^1] | title    |
| Rio            | [no](https://github.com/raphamorim/rio/issues/405)    | -        |

[^1]: To apply window rules for Konsole you need to use a partial pattern matching because Konsole modify
the title of the window to something like this: `[ASSIGNED_IDENTIFIER_BY_USER] — Konsole`. So you must
create a window rule with this syntax: `windowrule = [RULE], title:^[ASSIGNED_IDENTIFIER_BY_USER] —
Konsole$` or simply `windowrule = [RULE], title:^[ASSIGNED_TITLE_BY_USER]`
 

>[!NOTE]
>
> The identifier is needed by Hyprdrop to identify the dropped window and by Hyprland to apply window rules.


### Window Rules
For better experience you can add some window rules to your Hyprland config. This could create a
centered floating window with defined size.
```
windowrulev2 = float, class:^(alacritty_hyprdrop)$
windowrulev2 = center, class:^(alacritty_hyprdrop)$
windowrulev2 = size 1460 810, class:^(alacritty_hyprdrop)$
windowrulev2 = float, title:^(foot_hyprdrop)$
windowrulev2 = float, title:^(gnome-terminal_hyprdrop)
```

And some additional rules for TUI apps which is the same as above:
```
windowrulev2 = float, class:^(bottom_hyprdrop)$
windowrulev2 = center, class:^(bottom_hyprdrop)$
windowrulev2 = size 1460 810, class:^(bottom_hyprdrop)$
```
## Disclaimers

- This project is not affiliated with [Hyprland](https://github.com/hyprwm/Hyprland) or [hdrop](https://github.com/Schweber/hdrop).
- This project is in its early stages so it may not work as expected.
- I'm not a programmer so I don't know how to write good code.
