# Livesplit.Dolphin.SA2

An auto splitter for SA2B in Dolphin (NTSC-U).

## Usage:

Download the .wasm file from the releases tab.

Right click your livesplit -> Edit Layout -> + -> Control -> Auto splitting Runtime (If you don't see it, update your livesplit.)

Browse for your newly downloaded .wasm file.

Deactivate the current PC SA2B Autosplitter (right click -> edit splits -> deactivate)

Enjoy!


## Compilation

This auto splitter is written in Rust. In order to compile it, you need to
install the Rust compiler: [Install Rust](https://www.rust-lang.org/tools/install).

Afterwards install the WebAssembly target:
```sh
rustup target add wasm32-wasi --toolchain nightly
```

The auto splitter can now be compiled:
```sh
cargo b
```

The auto splitter is then available at:
```
target/wasm32-wasi/release/sonic_suggests_autosplitter.wasm
```

Make sure too look into the [API documentation](https://livesplit.org/asr/asr/) for the `asr` crate.

You can use the [debugger](https://github.com/CryZe/asr-debugger) while
developing the auto splitter to more easily see the log messages, statistics,
dump memory and more.
