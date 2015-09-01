ParView
=======

A particle visualization system. 

ParView creates high-quality, 3D visualizations and movies of spheres and other particles, as a visual aid to help in simple physical simulations.

Built in Rust, using GLFW through kiss3d, ParView is made to be fast, responsive, and very customizable.

There are many programs out there for viewing simulations — VMD, PyMol, etc. However, most of these are specifically for viewing *molecules*, and if your simulations don't particularly conform to this model, they can be difficult to view.

Parview allows fine-grained control over particle size, shape, and coloring, while using simple-to-create formats like `toml` and `json` for storing particle coordinates, viewer preferences, and color palettes.

# Usage

## Example

Here is an example of a packing of spherocylinders being created:

![](exampledata/spherocylinders.gif)

 Note: ParView outputs high-quality mpeg videos (created with `ffmpeg`); some noticeable quality was lost as part of the transition from mpeg to gif.

The conversion from `.mpg` to `.gif` was accomplished with this command:

```bash
ffmpeg -i spherocylinders.mpg -r 12 -vf scale=600:-1:flags=lanczos frames/spherocylinders%04d.png 
convert -delay 0.08333333 -layers removeDups -layers OptimizePlus -loop 0 frames/*.png -dither None exampledata/spherocylinders.gif
```

## File formats

Color palettes and configuration files are in [TOML](https://github.com/toml-lang/toml) format, with specific possibilities demonstrated in the [`exampleconfigs`](exampleconfigs) folder.

Data on particle positions and sizes are loaded from either a plain text JSON file, or gzipped JSON file. A minimal example of this is in `exampleconfigs/minimal.json`.

Note that particles are "named" with a *list* of strings. This is to allow for a hierarchy of naming. For example, if you had a protein, you might want the list of names to be `[residue_number, residue_type, element, atom_name]`, so that the user could easily switch between coloring by residue, coloring by residue type, coloring by atom element, etc.

## Binaries

There are two binaries produced by this library: `parviewer` and `pvrecord`. `parviewer` is for immediate viewing of particle trajectories, and `pvrecord` produces a `.mpeg` file.

### Parviewer

The binary `parviewer` is for simple viewing; it will show you the particle trajectories, and you can click and drag to rotate the image, or use the keybindings below.

#### Keybindings

| Key         | Function  |
|-------------|---|
| Q           | Quit |
| Space       | Pause playback |
| Comma, `,`  | Slow down playback |
| Period, `,` | Speed up playback  |
| F           | Reverse playback direction |
| Up          | Rotate the cube to a 45° angle, 30° from the plane |
| Down        | Rotate the cube to look directly along the z-axis |
| 1-9         | Choose which parts of a particle name are used for coloring |
| 0           | Eliminate all color by name |

Also note that clicking and dragging will rotate the view, and scrolling with the mouse will alter the zoom.

### PVRecord

The `pvrecord` binary is for producing movies. Run `pvrecord --help` for details. Note that only the `Q` button works; other keybindings are disabled, in order to prevent accidental clicks or button presses from modifying the video.

# Installation

## Dependencies

To install, you will need:

 * a nightly Rust compiler and the Cargo package manager.
    * At the moment, these are best obtained through [multirust](https://github.com/brson/multirust).
 * GLFW 3
   * on Arch Linux, this is just the `glfw` package.
   * For Ubuntu, see the `.travis.yml` file in this repository for an example of how to get GLFW 3; it will also require CMAKE.
 * FFMPEG

This library also depends on a large number of other Rust crates, but `cargo` will handle all that for you by default.

## Compiling

To compile everything, run these commands:

```bash
git clone https://github.com/wackywendell/parview.git
cd parview
multirust update  # Updates the rust compiler
multirust override nightly  # use the nightly compiler for this project
cargo build --release
```

The binaries will then be in the `parview/target/release` folder.
