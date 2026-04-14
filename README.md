# Blured  
A fancy wallpaper provider for Wayland.

<img width="2580" height="1080" alt="blured_showcase_1" src="https://github.com/user-attachments/assets/89dd5178-e2b8-414e-9b9b-1618b3b62a32" />

Blured supports wallpaper switching, slideshows, and toggleable per-wallpaper shader effects.  
The screenshot above shows a blur effect in action.


## Setup guide and example configurations
Blured reads its configuration from any `.toml` file in `~/.config/blured/`.

The `example_config/` folder contains example configurations.
* `example_config/defaults.toml` documents all configuration options and their default values.
* `example_config/slideshow.toml` is an example of a slideshow with 3 scenes.
* `example_config/custom_effect.toml` is an example of loading a custom effect shader.

1. Copy the example config:
```shell
cp -r example_config/ ~/.config/blured
```
2. Now run blured and pass the name of the config you would like to use as an argument. The config name is the file name without the extension.
```shell
# This example runs blured with the slideshow config
cargo run -r slideshow
```

Running blured without a config or with an invalid config will log at least a warning and display the built-in default config.  
The built-in config is the same as the one in `example_config/defaults.toml`.


## WIP Notice  
Many features are still a work in progress, most notably multi-monitor support.  
If you still want to check Blured out, you can run the project on any Wayland compositor that supports `wlr-layer-shell` (basically every compositor).  
It should display a hardcoded demo of the newest feature currently being worked on.

```shell
cargo run -r
```

### Roadmap
- [X] User configuration
- [ ] Better documentation and setup guide
  - [ ] Add documentation
  - [ ] Add setup guide
  - [ ] Add custom effect tutorial
- [ ] Multi monitor support and configuration
- [x] Command-line utility `bluredctl` for scene switching and effect toggling
- [ ] Adapters for popular compositors (Hyprland, Sway, Niri, ...) to expose information like "window count on active workspace", "is a window focused", or "focused window position" to blured
- [ ] More effects
- [ ] More image sources, for example "quote of the day", "space picture of the day", "animal picture of the day", or "picture based on the weather in your area"


## Gallery
* **Neuro - Abstract Galaxy Shader**
  <video src="https://github.com/user-attachments/assets/c88db0f6-1f1e-4b77-8132-29d0986047b8" loop muted/>

* **Blur - Toggleable Blur Shader** <br>
  Automatic background bluring using [bluredctl](https://github.com/rboeni2s/bluredctl) and [Hyprland](https://github.com/hyprwm/hyprland).
  <video src="https://github.com/user-attachments/assets/00bc0483-cca4-4398-8ad3-a62a11d175b4" loop muted/>


