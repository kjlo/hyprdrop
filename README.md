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
# Template
bind = $mainMod, [KEY], exec, hyprdrop [TERMINAL] -i [CHOOSEN_IDENTIFIER]
# Example
bind = $mainMod, U, exec, hyprdrop alacritty -i alacritty_hyprdrop
```
>[!NOTE]
>
> The argument identifier must be a unique name if you want to use as a separate application with
> special window rules.
Additionally, if you want to launch a TUI application:

```
# Template
bind = $mainMod, [KEY], exec, hyprdrop [TERMINAL] --identifier=[CHOOSEN_IDENTIFIER] --args=[TUI_CMD_AND_ARGS]
# Example
bind = $mainMod, I, exec, hyprdrop alacritty --identifier=bottom_hyprdrop --args=btm,-b
```
>[!NOTE]
>
> Check that for TUI applications it's not required to type the `-e` flag that most
> terminal emulators use when executing a command, this is implemented by Hyprdrop itself.

To launch Spotify:
```
# Template
bind = $mainMod, [CHOOSEN_KEY], exec, hyprdrop [GUI_APP] --identifier=[CHOOSEN_IDENTIFIER] --args=[GUI_APP_ARGS] --env=[ENVIRONMENT_VARIABLE]
# Example
bind = $mainMod, code:47, exec, hyprdrop spotify --identifier="Spotify Free" --args="--enable-features=UseOzonePlatform\,WaylandWindowDecorations,--ozone-platform=wayland" --env="LD_PRELOAD=/usr/lib/spotify-adblock.so"
```
>[!NOTE]
>
> Hyprdrop transform comma-separated values into space-separated values by default. If you want to
> ignore some value you must add the prefix `\`.

>[!NOTE]
>
> Spotify's class or title cannot be modified by the user so Hyprdrop uses the default
> `initial_title` to identify the app, in my case is `Spotify Free`.

You can check the `hyprdrop --help` command to see all the available options.

>[!WARNING]
>
> Hyprdrop was initially designed with TUI applications in mind. Theoretically, it should work with
> any GUI application or TUI application not fully supported (you are obligated to use the original
> class/title to identify the window).

### Supported Terminal Emulators
#### Fully Supported
The following list shows which terminal emulators are fully supported by Hyprdrop, this means you can
run every TUI application with this terminals and apply window rules.
| Terminal | Supported | Window Identifier (for Hyprland Config)|
|--------------- | ----- | -------- |
| Alacritty      | yes   | class    |
| Kitty          | yes   | class    |
| Wezterm        | yes   | class    |
| Gnome Terminal | yes   | title    |
| Foot           | yes   | title    |
| Konsole        | yes[^1] | title    |
| Rio            | [no](https://github.com/raphamorim/rio/issues/405)    | -        |
* This list is compiled based on testing conducted on those apps.

#### Partially Supported
| Terminal | Caveats |
|-------- | ---------------------------- |
| Spotify | Unable to apply window rules |
* This list is compiled based on testing conducted on those apps.

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
