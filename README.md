# Generator Labiryntu

Projekt na AiSD 2 (2023/24). Również dostępny na <https://j-markiewicz.github.io/maze>.

## Algorytmy i Struktury Danych

### Reprezentacja Labiryntu

Labirynt na potrzeby generacji i wizualizacji jest reprezentowany jako tablica pozycji o rozmiarze X * Y, czyli tablicy dwuwymiarowej. Przyleganie pozycji do siebie jest określane za pomocy indeksu dwóch pozycji - jeśli indeksy różnią się o 1 (pozycje są obok siebie) lub o X (pozycje są nad/pod sobą) to pozycje do siebie przylegają. Każda pozycja zawiera informacje o ścianach, które ma - przejście między pozycjami jest możliwe, jeśli nie ma pomiędzy nimi ściany.

### Generowanie Labiryntu

Do generowania labiryntu został użyty zmodyfikowany algorytm DFS, który w każdej iteracji otwiera przejście i przechodzi do losowej przylegającej pozycji (startując ze środka) lub jeśli wszystkie takie pozycje już zostały odwiedzone, to wraca do poprzedniej pozycji i próbuje ponownie. Algorytm się zakańcza kiedy wszystkie pozycje zostały odwiedzone. Dodatkowo, została dodana możliwość stworzenia "pokoi" w labiryncie, aby labirynt nie był acykliczny (pokoje to pozycje w labiryncie, które mają usunięte wszystkie ściany). Algorytm ten został wybrany, ponieważ jest dość prosty (zwłaszcza dla wybranej reprezentacji labiryntu), łatwy do zmodyfikowania, i generuje dobrze wyglądające labirynty.

## Kompilacja

Aby zbudować i uruchomić aplikację do lokalnego testowania, należy użyć [`cargo run --features dynamic,debug`](https://doc.rust-lang.org/cargo/commands/cargo-run.html). Pierwsza kompilacja zajmie kilka minut, ale kolejne powinny być znacznie szybsze. Obsługa profilowania przy użyciu [Tracy](https://github.com/wolfpld/tracy) może być włączona poprzez dodanie `feature` `profile` ([`cargo run --features dynamic,profile`](https://doc.rust-lang.org/cargo/commands/cargo-run.html)).

Aby zbudować aplikację z optymalizacjami, nalezy użyć [`cargo build --release`](https://doc.rust-lang.org/cargo/commands/cargo-build.html). Skompilowany plik będzie znajdował się w `./target/release/maze[.exe]`. Ten proces trwa kilka minut i nie jest zalecana do debugowania/testowania.

Aby zbudować `web-bg` dla platformy web (z pełnymi optymalizacjami), należy użyć [`cargo build --profile release-wasm --target wasm32-unknown-unknown`](https://doc.rust-lang.org/cargo/commands/cargo-build.html), stworzyć nowy katalog o nazwie `web` (`mkdir web`), a następnie użyć [`wasm-bindgen --out-name maze --out-dir target/wasm --target web target/wasm32-unknown-unknown/release-wasm/maze.wasm`](https://github.com/rustwasm/wasm-bindgen) i `cp target/wasm/maze_bg.wasm web/maze_bg.wasm` lub [`wasm-opt -O4 --output web/maze_bg.wasm target/wasm/maze_bg.wasm`](https://github.com/WebAssembly/binaryen), i skopiować do niego `index.html` i `target/wasm/web.js` jako `maze.js` (`cp index.html web/index.html` i `cp target/wasm/web.js web/maze.js`). Ten proces trwa kilka minut i nie jest zalecana do debugowania/testowania.

### Kompilacja dla WWW

Aby wypróbować aplikację w przeglądarce internetowej, postępuj zgodnie z powyższymi instrukcjami, aby zbudować ją dla platformy web, uruchom serwer HTTP (np. za pomocą `python -m http.server -d web`) i otwórz [`http://localhost:8000/`](http://localhost:8000/) w przeglądarce.

Aby umieścić program na stronie internetowej, należy udostępnić wygenerowane pliki `.js` i `.wasm` i dodać odpowiednie elementy do strony. Można użyć `index.html` jako szablonu i użyć `.github/workflows/build.yaml` jako przykład automatycznej kompilacji.

## Atrybucja

Oprócz bibliotek z Cargo, następujące dodatkowe zasoby są używane jako część tego projektu:

- Maze Cave (w `assets/maze/`):
  - Postać gracza bazująca na ["Reaper" - SamuelLee](https://samuellee.itch.io/reaper-animated-pixel-art) (`player-idle.png` i `player-walking.png`).
  - Pochodnia gracza z ["Cave Explorer" - SamuelLee](https://samuellee.itch.io/cave-explorer-animated-pixel-art) (`player-idle.png` i `player-walking.png`)
  - Teren z ["Textures" - PiiiXL](https://piiixl.itch.io/textures) (`cave-floor-1.png`, `cave-floor-2.png`, `cave-wall.png`, i `grass-*.png`)
- Inne:
  - Czcionka [Retro Pixel Thick](https://retro-pixel-font.takwolf.com/), używana na warunkach [Open Font License version 1.1](https://raw.githubusercontent.com/TakWolf/retro-pixel-font/0e90d12/LICENSE-OFL) w `assets/fonts/pixel.ttf`.
