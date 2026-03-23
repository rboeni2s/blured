# Blured  
A fancy wallpaper provider for Wayland.

<img width="2580" height="1080" alt="blured_showcase_1" src="https://github.com/user-attachments/assets/89dd5178-e2b8-414e-9b9b-1618b3b62a32" />

Blured supports wallpaper switching, slideshows, and toggleable per-wallpaper shader effects.  
The screenshot above shows a blur effect in action!

## WIP Notice  
Many features are still a work in progress, most notably user configuration.  
If you still want to check Blured out, you can run the project on any Wayland compositor that supports `wlr-layer-shell` (basically every compositor).

It should display two hardcoded wallpapers with two different stretch modes, switching every 30 seconds while toggling a nice animated blur effect every few seconds.

```shell
cargo run -r
```
