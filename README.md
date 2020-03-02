KAS 7GUIs
==========

[7GUIs](https://eugenkiss.github.io/7guis/) is a GUI programming benchmark.
This repository implements the benchmark's tasks via the [KAS] GUI.

[KAS]: https://github.com/kas-gui/kas

Installation and dependencies
----------------

Currently, KAS's only drawing method is [WebGPU](https://github.com/gfx-rs/wgpu-rs),
which requires DirectX 11/12, Vulkan or Metal.
In the future, there may be support for OpenGL and software rendering.

If you haven't already, install [Rust](https://www.rust-lang.org/), including
the *nightly* channel (`rustup toolchain install nightly`). Either make nightly
the default (`rustup default nightly`) or use `cargo +nightly ...` below.

A few other dependencies may require installation, depending on the system.
On Ubuntu:

```sh
sudo apt-get install build-essential git python3 cmake libxcb-shape0-dev libxcb-xfixes0-dev
```

Next, clone the repository:

```
git clone https://github.com/kas-gui/7guis
cd 7guis
```

Tasks
----

A brief list of the implemented tasks:

### Counter

```
cargo run --bin counter
```

A very simple push-button application.

### Temperature Converter

```
cargo run --bin temp-conv
```

An application to convert between Celsius and Fahrenheit temperatures.

Note: for now, one must press the *Enter* key to invoke the calculation.


Copyright and Licence
-------

This collection of examples is distributed under the "New BSD License".
See the COPYRIGHT file for details.
