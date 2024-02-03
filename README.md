# Hyprdrop

Rust implementation of [Hdrop](https://github.com/Schweber/hdrop).

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
> The argument class name has a simple requirement:
> - It must be different than the default class name of your terminal if you want to use as a separate application with special rules.

>[!WARNING]
>
> Hyprdrop currently supports only terminal applications. I have not tested it with other types yet.

### Window Rules
For better experience you can add some window rules to your hyprland config. This create a centered floating window with defined size.
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
