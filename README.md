KAS 7GUIs
==========

[7GUIs](https://eugenkiss.github.io/7guis/) is a GUI programming benchmark.
This repository implements the benchmark's tasks via the [KAS] GUI.

[KAS]: https://github.com/kas-gui/kas

Running the benchmark's tasks
----------------

If you haven't already, install Rust: <https://www.rust-lang.org/>

Cargo should take care of most dependencies, but note that:

-   [shaderc may require some setup](https://github.com/google/shaderc-rs#setup)
-   [WebGPU](https://github.com/gfx-rs/wgpu-rs) requires DirectX 11/12, Vulkan or
    Metal (in the future it may support OpenGL)

Next, clone the repository and run the tasks as follows:

```
git clone https://github.com/kas-gui/7guis
cd 7guis
cargo run --bin counter
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
