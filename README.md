# Hyprdrop

Rust implementation of [Hdrop](https://github.com/Schweber/hdrop).


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
This will create a binary in your `$HOME/.cargo/bin`. You must check that this address it's in your `$PATH`.


## Usage
The preferred way to use it is by adding it as a binding to your Hyprland config, like this:
```
bind = $mainMod, U, exec, hyprdrop alacritty --class alacritty_hyprdrop
```
Additionally, if you want to launch a TUI application with your terminal:
```
bind = $mainMod, I, exec, hyprdrop alacritty --class=bottom_hyprdrop --args btm,-b
```
>[!NOTE]
>
> The argument class name must be a unique name if you want to use as a separate application with
> special window rules.

>[!WARNING]
>
> Hyprdrop was initially designed with TUI applications in mind. Theoretically, it should work with
> any GUI application. However, one consideration is that it is not usable with the `args` flag, as
> it is specifically designed for terminal emulators.

### Window Rules
For better experience you can add some window rules to your Hyprland config. This create a centered
floating window with defined size.
```
windowrulev2 = float, class:^(alacritty_hyprdrop)$
windowrulev2 = center, class:^(alacritty_hyprdrop)$
windowrulev2 = size 1460 810, class:^(alacritty_hyprdrop)$
```

And some additional rules for TUI apps:
```
windowrulev2 = float, class:^(bottom_hyprdrop)$
windowrulev2 = center, class:^(bottom_hyprdrop)$
windowrulev2 = size 1460 810, class:^(bottom_hyprdrop)$
```
