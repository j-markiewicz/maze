# Generator Labiryntu

Projekt na AiSD 2 (2023/24).
Kod na <https://github.com/j-markiewicz/maze>.
Program do pobrania na <https://github.com/j-markiewicz/maze/releases> i dostępny online na <https://j-markiewicz.github.io/maze>.

## Działanie Programu

### Wejścia Programu

Program poprzez menu (TAB lub ESC) pobiera szerokość i wysokość labiryntu oraz ilość "pokoi", czyli pozycji w labiryncie całkowicie otwartych (istnienie takich pokoi powoduje, że labirynt nie jest acykliczny). Dodatkowo można ustalić tendencyjność kierunkową labiryntu, co powoduje generacje labiryntu z innym wyglądem.

Minimalna szerokość lub wysokość labiryntu to 3, maksymalna to 100. Nie ma limitu co do ilości pokoi, ale im więcej jest pokoi, tym większa szansa, że wygenerują się dwa (lub więcej) pokoje na tym samym miejscu. Pierwszy pokój zawsze jest generowany na pozycji startowej (w środku labiryntu).

Struktura przechowywująca te parametry znajduje się w [`src/algorithms.rs`](https://github.com/j-markiewicz/maze/blob/main/src/algorithms.rs#L22-L33).

### Reprezentacja Labiryntu

Labirynt na potrzeby generacji i wizualizacji jest reprezentowany jako tablica pozycji o rozmiarze X * Y, czyli tablicy dwuwymiarowej. Przyleganie pozycji do siebie jest określane za pomocy indeksu dwóch pozycji - jeśli indeksy różnią się o 1 (pozycje są obok siebie) lub o X (pozycje są nad/pod sobą) to pozycje do siebie przylegają. Każda pozycja zawiera informacje o ścianach, które ma - przejście między pozycjami jest możliwe, jeśli nie ma pomiędzy nimi ściany.

Struktura przechowywująca labirynt znajduje się w [`src/maze.rs`](https://github.com/j-markiewicz/maze/blob/main/src/maze.rs#L37-L44), gdzie na poszczególnych [pozycjach](https://github.com/j-markiewicz/maze/blob/main/src/maze.rs#L462-L466) znajdują się [kafelki](https://github.com/j-markiewicz/maze/blob/main/src/maze.rs#L220-L221) z informacjami o otwartych i zamkniętych ścianach.

### Generowanie Labiryntu

Do generowania labiryntu został użyty zmodyfikowany algorytm DFS, który w każdej iteracji otwiera przejście i przechodzi do losowej przylegającej pozycji (startując ze środka) lub jeśli wszystkie takie pozycje już zostały odwiedzone, to wraca do poprzedniej pozycji i próbuje ponownie. Algorytm się zakańcza kiedy wszystkie pozycje zostały odwiedzone. Dodatkowo, została dodana możliwość stworzenia "pokoi" w labiryncie, aby labirynt nie był acykliczny (pokoje to pozycje w labiryncie, które mają usunięte wszystkie ściany). Algorytm ten został wybrany, ponieważ jest dość prosty (zwłaszcza dla wybranej reprezentacji labiryntu), łatwy do zmodyfikowania, i generuje dobrze wyglądające labirynty.

Funkcje generujące labirynt znajdują się w pliku `src/algorithms.rs`: [`gen_maze`](https://github.com/j-markiewicz/maze/blob/main/src/algorithms.rs#L151-L212) generuje korytarze labiryntu, a [`gen_rooms`](https://github.com/j-markiewicz/maze/blob/main/src/algorithms.rs#L214-L245) dodaje pokoje. Labirynt jest dodadkowo przetwarzany przez funkcje z `src/maze.rs` - [`prepare_maze`](https://github.com/j-markiewicz/maze/blob/main/src/maze.rs#L566-L582), która przygotowywuje tablice do generacji przez `gen_maze` oraz [`adjust_maze_textures`](https://github.com/j-markiewicz/maze/blob/main/src/maze.rs#L584-L630), która poprawia wygląd kątów w labiryncie po generacji.

### Szukanie Wyjścia z Labiryntu

Po generacji labiryntu stworzone jest drzewo najkrótszych ścieżek ([*shortest-path tree*](https://en.wikipedia.org/wiki/Shortest-path_tree)) przy użyciu algorytmu [Dijkstry](https://en.wikipedia.org/wiki/Dijkstra's_algorithm). Podczas wizualizacji labiryntu to drzewo jest używane aby bardzo szybko (bez ponownego przeszukania labiryntu) znaleźć najkrótszą ścieżkę z wybranej pozycji do wyjścia. Drzewo jest reprezentowane w tablicy z indeksami użytymi jako "wskaźniki" do rodziców.

Funkcja szukająca wyjścia z labiryntu znajduje się w pliku [`src/algorithms.rs`](https://github.com/j-markiewicz/maze/blob/main/src/algorithms.rs#L368-L442).

Struktury przechowywujące drzewo najkrótszych ścieżek znajdują się w pliku `src/algorithms.rs`, nieposortowana wersja przystosowana do szybkiego dodawania wierzchołków nazywa się [Tree](https://github.com/j-markiewicz/maze/blob/main/src/algorithms.rs#L309-L353), a posortowana wersja przystosowana do szybkiego szukania elementów nazywa się [SortedTree](https://github.com/j-markiewicz/maze/blob/main/src/algorithms.rs#L247-L307). `Tree` jest konwertowane w `SortedTree` przez [`SortedTree::new`](https://github.com/j-markiewicz/maze/blob/main/src/algorithms.rs#L254-L286).

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
