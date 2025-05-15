# rust-life

This is a sparse implementation of Conway's Game of Life in Rust. I mostly wrote
this as a toy project to play around with Rust TUI applications. It is kind of
fascinating to watch though.

## Installation

You will need the Rust toolchain (`cargo` etc.) to build the project. Once you
have that, you can build and run the usual way (`cargo build`, `cargo run`,
etc.)

## Use

The program accepts two command line options.

- `-t` which accepts an argument to specify the simulation timestep in
  milliseconds. Defaults to `100`.
- `-p` which accepts an argument to specify an initial pattern of cells. The
  available patterns are: glider, beacon, blinker, and random.

When the TUI is active you can move the viewport, pause the simulation, or
manually add or remove cells. The keybindings are summarized below:

- `q`/`Esc`: Quit
- `Space`: Play/Pause
- `←↓↑→`/`hjkl`: Move viewport
- `o`: Center viewport on the origin
- `wasd`: Move cursor
- `e`: Toggle cell under cursor
- `c`: Center cursor in viewport

## Technical Details

Internally, the cells are stored in a hashmap to allow the data structure to
expand and contract dynamically. When a cell becomes alive, any missing
neighbors are added to the hashmap, and dead cells with no living neighbors are
removed from the hashmap. The number of neighbors is cached in the cell data
structure and updated when cells change stated. The idea is to avoid iterating
over and checking many dead cells when dealing with large maps. There are other
ways to make large simulations more efficient (e.g. we could dynamically add and
remove "chunks" of the life grid) but this seemed like a simple and elegant way.

## To-Do

- CLI option to start paused, start paused automatically if pattern is blank.
- Explore parallelism for simulation.
- Add "splatter" feature to randomly add cells to visible area.
- Rewind feature.

