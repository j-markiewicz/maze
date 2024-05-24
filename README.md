# Generator Labiryntu

Projekt na AiSD 2 (2023/24).

## Algorytmy i Struktury Danych

### Reprezentacja Labiryntu

Labirynt na potrzeby generacji i wizualizacji jest reprezentowany jako tablica pozycji o rozmiarze X * Y, czyli tablicy dwuwymiarowej. Przyleganie pozycji do siebie jest określane za pomocy indeksu dwóch pozycji - jeśli indeksy różnią się o 1 (pozycje są obok siebie) lub o X (pozycje są nad/pod sobą) to pozycje do siebie przylegają. Każda pozycja zawiera informacje o ścianach, które ma - przejście między pozycjami jest możliwe, jeśli nie ma pomiędzy nimi ściany.

### Generowanie Labiryntu

Do generowania labiryntu został użyty zmodyfikowany algorytm DFS, który w każdej iteracji otwiera przejście i przechodzi do losowej przylegającej pozycji (startując ze środka) lub jeśli wszystkie takie pozycje już zostały odwiedzone, to wraca do poprzedniej pozycji i próbuje ponownie. Algorytm się zakańcza kiedy wszystkie pozycje zostały odwiedzone. Dodadkowo, została dodana możliwość stworzenia "pokoji" w labiryncie, aby labirynt nie był acykliczny (pokoje to pozycje w labiryncie, które mają usunięte wszystkie ściany). Algorytm ten został wybrany, ponieważ jest dość prosty (zwłaszcza dla wybranej reprezentacji labiryntu), łatwy do zmodyfikowania, i generuje dobrze wyglądające labirynty.

## Building

To build and run `web-bg` as a regular application for local testing, run [`cargo run --features dynamic,debug`](https://doc.rust-lang.org/cargo/commands/cargo-run.html). Note that while the first compilation will take a few minutes, subsequent builds should be much faster. Profiling support for [Tracy](https://github.com/wolfpld/tracy) can be enabled by adding the `profile` feature ([`cargo run --features dynamic,profile`](https://doc.rust-lang.org/cargo/commands/cargo-run.html)).

To build `web-bg` as a regular application, run [`cargo build --release`](https://doc.rust-lang.org/cargo/commands/cargo-build.html). The compiled binary will be located in `./target/release/web-bg[.exe]`. This build takes a few minutes, and is not recommended for debugging/testing/development.

To build `web-bg` for the web (with full optimizations), run [`cargo build --profile release-wasm --target wasm32-unknown-unknown`](https://doc.rust-lang.org/cargo/commands/cargo-build.html), create a new directory named `web` (`mkdir web`), then run [`wasm-bindgen --out-name web --out-dir target/wasm --target web target/wasm32-unknown-unknown/release-wasm/web-bg.wasm`](https://github.com/rustwasm/wasm-bindgen) and optionally [`wasm-opt -Oz --output web/web_bg.wasm target/wasm/web_bg.wasm`](https://github.com/WebAssembly/binaryen), then copy `index.html` and `target/wasm/web.js` as `background.js` into it (`cp index.html web/index.html` and `cp target/wasm/web.js web/background.js`). This build takes a few minutes, and is not recommended for debugging/testing/development.

### Web builds

To try out `web-bg` in a web browser follow the instructions above to build it for the web, start an HTTP server (e.g. with `python -m http.server -d web`) and open [`http://localhost:8000/`](http://localhost:8000/) in a browser.

When deploying `web-bg` on a website, serve the generated `.js` and `.wasm` files and add the appropriate elements to your website. You can use `index.html` as a template. See `.github/workflows/build.yaml` for an example of an automatic build of `web-bg`.

## Usage on the web

See `index.html` for an example of usage.

`web-bg` needs a `canvas` element with id `background` to render to.
The size of that element will be set to match the size of its parent by `web-bg`.

`web-bg` takes keyboard, mouse, and touchscreen input from its canvas element.
Websites should provide a way for the user to focus on that element, for example by clicking/tapping on it or via a global keyboard shortcut.

`web-bg` dispatches JavaScript events to the `window` during various phases of execution:

- `web-bg-load` when the application starts executing
- `web-bg-init` when the application has initialized
- `web-bg-start` when the application is fully ready for usage (`web-bg`'s canvas should be hidden until this event is received)
- `web-bg-panic` if the application panics (`web-bg`'s canvas should be hidden when this event is received)

### Logging on the web

If the `console_log` feature is enabled and you compile `web-bg` for the web, log messages will be logged to the console and tracing spans will be measured using the Performance API, at the expense of degraded application performance.
The `console_log` feature does nothing when *not* compiling for the web.

## Attribution

In addition to Cargo dependencies, the following additional resources are used as part of this project:

- Maze Cave (in `assets/maze/`):
  - Player character based on ["Reaper" by SamuelLee](https://samuellee.itch.io/reaper-animated-pixel-art) (`player-idle.png` and `player-walking.png`)
  - Player's torch from ["Cave Explorer" by SamuelLee](https://samuellee.itch.io/cave-explorer-animated-pixel-art) (`player-idle.png` and `player-walking.png`)
  - Cave tiles based on ["Textures" by PiiiXL](https://piiixl.itch.io/textures) (`cave-floor-1.png`, `cave-floor-2.png`, and `cave-wall.png`)
  - Food from ["Pixel Food" by ghostpixxells](https://ghostpixxells.itch.io/pixelfood) (`food.png` and `plate.png`)
- Miscellaneous:
  - The [Roboto font](https://fonts.google.com/specimen/Roboto), used under the terms of the [Apache 2.0 license](https://www.apache.org/licenses/LICENSE-2.0) in `assets/fonts/roboto.ttf` and `assets/fonts/roboto-bold.ttf`
  - The [Retro Pixel Thick font](https://retro-pixel-font.takwolf.com/), used under the terms of the [Open Font License version 1.1](https://raw.githubusercontent.com/TakWolf/retro-pixel-font/0e90d12/LICENSE-OFL) in `assets/fonts/pixel.ttf`
  - [`github-markdown-css`](https://github.com/sindresorhus/github-markdown-css), used under the terms of [the MIT license](./about.hbs#this-document) for styling in `about.hbs` (and the html file generated from it)
