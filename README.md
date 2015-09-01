ParView
=======

A particle visualization system. 

ParView creates high-quality, 3D visualizations and movies of spheres and other particles, as a visual aid to help in simple physical simulations.

Built in Rust, using GLFW through kiss3d, ParView is made to be fast, responsive, and very customizable.

There are many programs out there for viewing simulations — VMD, PyMol, etc. However, most of these are specifically for viewing *molecules*, and if your simulations don't particularly conform to this model, they can be difficult to view.

Parview allows fine-grained control over particle size, shape, and coloring, while using simple-to-create formats like `toml` and `json` for storing particle coordinates, viewer preferences, and color palettes.

## Example

Here is an example of a packing of spherocylinders being created:

![](exampledata/spherocylinders.gif)

ParView outputs high-quality mpeg videos (created with `ffmpeg`); some noticeable quality was lost as part of the transition from mpeg to gif.

The conversion from `.mpg` to `.gif` was accomplished with this command:

```bash
ffmpeg -i spherocylinders.mpg -r 12 -vf scale=600:-1:flags=lanczos frames/spherocylinders%04d.png 
convert -delay 0.08333333 -layers removeDups -layers OptimizePlus -loop 0 frames/*.png -dither None exampledata/spherocylinders.gif

```
