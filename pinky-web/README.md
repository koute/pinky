This is the Web frontend for Pinky.

It's mostly meant as a demo for Rust's WebAssembly capabilities
and the [stdweb] project.

[stdweb]: https://github.com/koute/stdweb

[![Become a patron](https://koute.github.io/img/become_a_patron_button.png)](https://www.patreon.com/koute)

### See it in action

  * A version [compiled with Rust's native WebAssembly backend] (recommended!).
  * A version [compiled to WebAssembly with Emscripten].
  * A version [compiled to asm.js with Emscripten].

[compiled with Rust's native WebAssembly backend]: https://koute.github.io/pinky-web
[compiled to WebAssembly with Emscripten]: https://koute.github.io/pinky-web-webasm-emscripten
[compiled to asm.js with Emscripten]: https://koute.github.io/pinky-web-asmjs-emscripten

### Building (using Rust's native WebAssembly backend)

1. Install newest nightly Rust:

       $ curl https://sh.rustup.rs -sSf | sh

2. Install WebAssembly target:

       $ rustup target add wasm32-unknown-unknown

3. Install [cargo-web]:

       $ cargo install -f cargo-web

4. Build it:

       $ cargo web start --target-webasm --release

5. Visit `http://localhost:8000` with your browser.

[cargo-web]: https://github.com/koute/cargo-web

### Building for other backends

Replace `--target-webasm` with `--target-webasm-emscripten` or `--target-asmjs-emscripten`
if you want to build it using another backend. You will also have to install the
corresponding targets with `rustup` - `wasm32-unknown-emscripten` and `asmjs-unknown-emscripten`
respectively.
