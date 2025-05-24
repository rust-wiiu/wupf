# Wii U Plugin Framework

The **Wii U Plugin Framework (WUPF)** simplifies the creation of WUPS Homebrew applications. It builds upon the [`wups`](https://github.com/rust-wiiu/wups) library by offering an intuitive, stateful approach to plugin development with commonly needed functionalities.

```rust
use wupf::{PluginHandler, Plugin, hook_plugin};

#[derive(PluginHandler)]
struct MyPlugin {
    value: u32
}

hook_plugin!(MyPlugin);
impl Plugin for MyPlugin {
    fn on_init() -> Self {}
    fn on_deinit(&mut self) {}
    fn on_start(&mut self) {}
    fn on_exit(&mut self) {}
}
```

### Installation

To integrate WUPF into your project, add it via `cargo`:

```bash
cargo add --features derive --git https://github.com/rust-wiiu/wupf wupf
```
