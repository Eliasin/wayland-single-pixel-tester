# wayland-single-pixel-tester

Super small program that creates an `xdg_toplevel` surface and attaches a buffer created using the `wp_single_pixel_buffer_manager_v1`. It's super barebones and janky but it works as a quick tester.
This is a modified version of the `single_window` example from `Smithay/wayland-rs`'s examples.

## Building

``` rust
cargo build
```

