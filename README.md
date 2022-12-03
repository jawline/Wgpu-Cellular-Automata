## GPU Accelerated Cellular Automata

This is an implementation of a GPU accelerated 3D cellular automata which can
be used to implement common cellular automata rulesets in 2D and 3D (e.g,
Conway's game of life). 

<p float="left">
  <img src="/screenshots/one.png" width="49%" />
  <img src="/screenshots/two.png" width="49%" />
  <img src="/screenshots/three.png" width="49%" />
</p>

### Usage

cargo run --release

Hit R to re-seed the scene.
Use WASD to navigate.

### Changing Rulesets

The ruleset is implemented through a small DSL in Rust. For an example, view
the `conways_game_of_life` function in `src/automata_dsl.rs`.
