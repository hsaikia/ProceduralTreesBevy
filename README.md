# Procedural Trees 

This bevy plugin produces procedurally generated fractal trees using very simple rules. Watch [this video](https://youtu.be/1eHpa3nqhus?si=LMJgUP-6VwcG7qIT) for a description of the rules.

![screenshot](screenshot.png)

## How to run

You must have rustc and cargo installed (see [here](https://www.rust-lang.org/tools/install)). Then clone the repo and from the root, simply run this command

```
cargo run --release
```

To run on the web, you must set an environment variable (see [here](https://bevy-cheatbook.github.io/platforms/wasm.html))

```
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-server-runner
```

Then run 

```
cargo run --target wasm32-unknown-unknown
```
