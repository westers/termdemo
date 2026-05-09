# termdemo

A love letter to the demoscene, running entirely in your terminal. 68 real-time visual effects rendered at 60fps using Unicode half-block characters for square pixel output. No GPU, no dependencies beyond a terminal emulator.

## Quick Start

```bash
# Build
cargo build --release

# Run in autoplay mode (sit back and watch)
./target/release/termdemo

# Run in interactive mode (browse effects manually)
./target/release/termdemo --interactive

# Or via cargo (-- separates cargo args from program args)
cargo run --release
cargo run --release -- --interactive
```

## Controls

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Space` | Pause / resume |
| `Tab` | Toggle autoplay / interactive mode |
| `n` / `Right` | Next effect |
| `p` / `Left` | Previous effect |
| `f` | Hold current scene (prevent auto-advance) |
| `h` | Toggle HUD overlay |
| `Up` / `Down` | Adjust current effect parameter |
| `[` / `]` | Select previous / next parameter |
| `1`-`9` | Jump to effect 1-9 |

## Requirements

- Rust 1.56+ (2021 edition)
- A terminal with true-color (24-bit) support
- Recommended: 80x24 minimum, larger terminals look better

## The Effects

Each entry includes a **Technique** (what's being computed and how) and, where applicable, a **Heritage** (the demoscene or computer-graphics history behind it).

### Act 1 -- Classic Patterns

#### 1. Plasma
**Technique** -- Sum of several sine waves over (x, y, t) mapped through HSV; classic fragment-shader-style per-pixel evaluation.
**Heritage** -- The quintessential demoscene effect, popularized on the Amiga in the late 1980s. Every demo group had their own variant.

#### 2. Moire
**Technique** -- Two overlapping concentric circle patterns at slightly different frequencies produce shimmering interference fringes.
**Heritage** -- Named after the French textile weaving technique; a staple of early computer graphics demonstrations.

#### 3. Kaleidoscope
**Technique** -- Polar-coordinate fold (mod by `2π/n`) reflects a procedural source pattern across `n` axes of symmetry.
**Heritage** -- Emulates the optical toy invented by David Brewster in 1816.

#### 4. Shadebobs
**Technique** -- Additive light blobs traced into a persistent canvas along Lissajous-like paths, leaving glowing trails.
**Heritage** -- A signature effect of early-1990s Amiga demos, exploiting the blitter for fast screen compositing.

#### 5. Copper Bars
**Technique** -- Horizontal gradient bars at sine-modulated vertical positions, additively blended.
**Heritage** -- Named after the Amiga's Copper coprocessor, which could change palette registers mid-scanline. A defining visual of the Amiga demoscene.

#### 6. Raster Bars
**Technique** -- Per-scanline color manipulation with continuous palette interpolation across the full frame.
**Heritage** -- Originally a C64 technique using raster interrupts to change border and background colors mid-scanline.

#### 7. Copper Flag
**Technique** -- Copper-bar palette with sine-wave horizontal displacement of each scanline, creating a waving cloth.
**Heritage** -- Combines the copper-bar color trick with cloth-like wave animation.

#### 8. Kefrens Bars
**Technique** -- Vertical bars rendered with per-row horizontal offset, producing an interleaved weaving curtain.
**Heritage** -- Named after the legendary Danish demo group Kefrens, whose Amiga demos made this iconic.

#### 9. Truchet
**Technique** -- Grid of randomly oriented quarter-circle tiles whose endpoints align across cell boundaries to produce flowing maze-like patterns.
**Heritage** -- Based on tilework by Sebastien Truchet (1704); rediscovered by Cyril Stanley Smith in 1987.

#### 10. Interference
**Technique** -- Sum of two radial cosine waves from separate point sources, producing high-contrast bright/dark fringes.
**Heritage** -- Simulates Thomas Young's 1801 double-slit experiment that proved the wave nature of light.

### Act 2 -- Heat & Motion

#### 11. Fire
**Technique** -- Each pixel is replaced by a weighted average of the three pixels below it, multiplied by a cooling factor; hot embers are seeded along the bottom row each frame.
**Heritage** -- A defining real-time effect of early-1990s PC demos, with variants on every platform.

#### 12. Twister
**Technique** -- A rotating rectangular bar drawn as horizontal bands, each at a sine-displaced x position; only 1D math per scanline.
**Heritage** -- A signature effect of 1990s Amiga and PC demos.

#### 13. Tunnel
**Technique** -- Polar (angle, distance) lookup tables precomputed once; each frame texture-samples a procedural pattern with time offset.
**Heritage** -- First popularized in PC demos around 1993; became one of the most recognizable demoscene effects.

#### 14. Dot Tunnel
**Technique** -- Concentric rings of dots projected through a perspective transform with z animated to simulate forward motion.
**Heritage** -- A lighter variant of the solid tunnel, popular on 8-bit and 16-bit platforms where fill rate was limited.

#### 15. Rotozoom
**Technique** -- Inverse 2D affine transform per pixel: each screen point maps to rotated, scaled source coordinates.
**Heritage** -- A staple of the Amiga and Atari ST demoscenes.

#### 16. Lightning
**Technique** -- Recursive midpoint displacement generates jagged bolt paths; flash illumination is a uniform brightness boost on strike events.
**Heritage** -- The midpoint-displacement technique was borrowed from fractal terrain generation.

#### 17. Lava Lamp
**Technique** -- Metaball implicit field (sum of inverse-distance-squared blobs) thresholded with warm color mapping; blobs drift on slow sinusoids.
**Heritage** -- Visually emulates the 1963 invention by Edward Craven Walker.

### Act 3 -- 3D Geometry

#### 18. Starfield
**Technique** -- 3D point cloud whose z perpetually decreases toward the camera; perspective division yields warp lines.
**Heritage** -- One of the very first demo effects, dating back to the C64 era.

#### 19. Galaxy
**Technique** -- Particle system on logarithmic-spiral arms with random scatter, rendered as additive blobs.
**Heritage** -- Inspired by density-wave theory explaining real galactic structure.

#### 20. Dot Sphere
**Technique** -- Points distributed on a unit sphere via the Fibonacci spiral method, rotated and projected with depth-based brightness.
**Heritage** -- Descended from dot-based 3D effects on 8-bit platforms.

#### 21. Boing Ball
**Technique** -- A textured sphere with red/white meridian quadrants, bouncing under gravity in a purple grid room with shadow projection.
**Heritage** -- The iconic 1984 Amiga Boing Ball demo that introduced the platform at CES and became its unofficial mascot.

#### 22. Filled Vector
**Technique** -- Flat-shaded rotating icosahedron with painter's-algorithm depth sorting and per-face Lambertian shading.
**Heritage** -- Represents the leap from wireframe to solid 3D in demos circa 1990-1992.

#### 23. Morph
**Technique** -- Linear interpolation of corresponding vertex pairs between sphere, cube, and torus point clouds, with rotating projection.
**Heritage** -- 3D morphing went mainstream in films like Terminator 2 (1991) and quickly became a demo staple.

#### 24. Glenz
**Technique** -- Transparent overlapping convex polyhedra rendered with additive blending and no depth sort.
**Heritage** -- Named after the "Glenz vector" style popularized by Future Crew in landmark PC demos like Second Reality (1993).

#### 25. Lissajous 3D
**Technique** -- 3D parametric curve from three orthogonal sines with non-commensurate frequencies, accumulated into a fading trail.
**Heritage** -- Studied in 1857 by Jules Antoine Lissajous using tuning forks and mirrors.

#### 26. Torus Knot
**Technique** -- A (p, q) torus-knot parametric curve sampled densely and projected with perspective; rendered as a glowing two-pass dot trail.
**Heritage** -- Studied in mathematical knot theory; common in 2000s demos and screensavers.

#### 27. Wireframe
**Technique** -- 3D model rotation with back-face culling and Bresenham-drawn edges.
**Heritage** -- The original mode of real-time 3D graphics, dating to Ivan Sutherland's Sketchpad (1963).

#### 28. Phong
**Technique** -- SDF ray-march of a single torus shaded with per-pixel ambient + Lambertian diffuse + Blinn-Phong specular and a rim term; material hue cycles slowly.
**Heritage** -- Phong illumination was published by Bui Tuong Phong in 1975 and remained the default real-time shading model for two decades. The "shiny spinning teapot/torus" was a demoscene status symbol of 3D rendering ability.

#### 29. Chrome
**Technique** -- Same SDF torus, but lit purely by environment-mapped reflection (procedural sky gradient + sun + warm checker ground), sampled by the reflection vector with a Fresnel edge tint.
**Heritage** -- Chrome objects were demoscene flexes throughout the 1990s -- they signaled that the coder had real-time normal computation and texture sampling working at full speed.

#### 30. Klein Bottle
**Technique** -- Figure-8 immersion of the Klein bottle parameterized over (u, v); 90×30 surface samples drawn with two-pass glow + bright dots, full 3D rotation.
**Heritage** -- The Klein bottle (Felix Klein, 1882) is a non-orientable closed surface that cannot be embedded in 3D space without self-intersection -- a topology curiosity that has fascinated mathematicians and graphics programmers alike.

#### 31. Cube Field
**Technique** -- Flying through an infinite grid of flat-shaded cubes; each cube z-fades into fog as the viewer position advances along z.
**Heritage** -- Inspired by the Flash game "Cubefield" (2006), adapted as a demoscene fly-through.

#### 32. Wolfenstein
**Technique** -- DDA raycasting against a 2D grid; per-column wall-slice height computed from ray distance, with simple texturing.
**Heritage** -- John Carmack's algorithm in Wolfenstein 3D (1992) revolutionized real-time pseudo-3D for games.

#### 33. Raymarcher
**Technique** -- Sphere-tracing signed distance fields with smooth-min unions; Phong shading + checker floor + distance fog.
**Heritage** -- SDF raymarching was pioneered by demosceners (notably Inigo Quilez) for stunning 4KB intros.

#### 34. Terrain
**Technique** -- Heightmap flyover via column-based raycasting (the "Voxel Space" algorithm).
**Heritage** -- Inspired by the Comanche engine (NovaLogic, 1992), which rendered voxel landscapes in real-time on 386 PCs.

#### 35. Voxel Landscape
**Technique** -- For each screen column, walk the heightmap from far to near, painting upward to a maximum projected height -- the same algorithm Comanche used.
**Heritage** -- Made convincing real-time outdoor 3D viable on consumer hardware before dedicated GPUs.

### Act 4 -- Fractals

#### 36. Julia
**Technique** -- Per-pixel iteration of `z = z² + c` with a fixed `c` animated to trace a path through parameter space; iteration count maps to color.
**Heritage** -- Studied by Gaston Julia in the 1910s; each Mandelbrot point corresponds to a unique Julia set.

#### 37. Fractal Zoom
**Technique** -- Smooth perpetual zoom into the Mandelbrot set's Seahorse Valley, with adaptive max-iterations as zoom deepens.
**Heritage** -- A demonstration of Benoit Mandelbrot's 1980 discovery and the infinite self-similar detail of the set.

#### 38. Sierpinski
**Technique** -- Chaos game: a moving point repeatedly jumps halfway toward a randomly chosen vertex of a triangle, and the Sierpinski gasket emerges from the orbit's invariant measure.
**Heritage** -- Discovered by Waclaw Sierpinski in 1915; the chaos-game variant by Michael Barnsley.

#### 39. Apollonian
**Technique** -- Recursive circle packing using Descartes' Circle Theorem and the curvature-mirror identity: each new circle is the "other" Soddy circle of three mutually tangent ancestors.
**Heritage** -- Named after Apollonius of Perga (~250 BC), who first studied the problem of finding circles tangent to three given circles.

#### 40. Attractor
**Technique** -- Numerical integration of a chaotic dynamical system (Aizawa, Lorenz, or Thomas, chosen randomly per show) with point accumulation into an additive buffer and Reinhard tone-mapping.
**Heritage** -- Strange attractors became the iconic visualization of chaos theory after Edward Lorenz published his weather model in 1963.

### Act 5 -- Simulations

#### 41. Metaballs
**Technique** -- Implicit surface field summed from inverse-distance-squared blobs, threshold-rendered with smooth contour shading.
**Heritage** -- Invented by Jim Blinn in 1982 as "blobby molecules"; a signature 1990s demo effect and a 2000s Flash-web staple.

#### 42. Voronoi
**Technique** -- For each pixel, find the nearest seed point and color by its index, partitioning the plane into convex cells.
**Heritage** -- Named after Georgy Voronoy (1908); appears in domains from cell biology to airport coverage.

#### 43. Reaction-Diffusion
**Technique** -- Gray-Scott model: two chemicals `u` and `v` diffuse and react with parameters (F, k); spots, stripes, and labyrinths emerge spontaneously.
**Heritage** -- Alan Turing proposed reaction-diffusion as the basis of biological morphogenesis in 1952.

#### 44. Fluid Simulation
**Technique** -- Jos Stam's stable-fluids method: diffuse, advect with semi-Lagrangian backtrace, then project to enforce divergence-free velocity.
**Heritage** -- Stam's 1999 paper made real-time fluid simulation practical for games and demos.

#### 45. Cloth Simulation
**Technique** -- Verlet integration of point masses connected by distance constraints, iterated for stability; pinned vertices, gravity, and wind drive motion.
**Heritage** -- Method popularized by Thomas Jakobsen in Hitman: Codename 47 (2001).

#### 46. Water
**Technique** -- 2D height field where each cell averages its four neighbors and dampens, producing expanding wave fronts when disturbed.
**Heritage** -- A classic 1990s DOS effect.

#### 47. Fountain
**Technique** -- Particle system with gravity, randomized initial velocity, and lifetime-based color/alpha fade.
**Heritage** -- Particle systems were formalized by Bill Reeves at Lucasfilm for the Genesis effect in Star Trek II (1982).

#### 48. Boids
**Technique** -- Three local rules -- separation, alignment, cohesion -- applied to each agent based on neighbors within a radius; emergent flocking from local interactions.
**Heritage** -- Craig Reynolds' 1986 algorithm.

#### 49. Cellular Automata
**Technique** -- Brian's Brain: a 3-state CA (off / on / dying) with neighbor-count transition rules, producing chaotic gliders and oscillators.
**Heritage** -- A variation on the cellular-automaton framework pioneered by John von Neumann.

#### 50. Game of Life
**Technique** -- Conway's 2-state CA: a cell lives if it has 2-3 living neighbors; dead cells with exactly 3 living neighbors are born.
**Heritage** -- John Conway, 1970; Turing-complete in the limit.

### Act 6 -- Natural & Atmospheric

#### 51. Aurora Borealis
**Technique** -- Stacked sine-wave curtains with vertical falloff, additively blended in green/teal/violet over a deep-sky background.
**Heritage** -- Real aurorae are caused by solar wind particles exciting atmospheric gases.

#### 52. Rain
**Technique** -- Layered parallax raindrops with depth-based velocity; splash particles spawn on impact; lightning events trigger a full-screen flash.
**Heritage** -- Combines particle systems, parallax scrolling, and procedural flash events.

#### 53. Snowfall
**Technique** -- Three depth layers of drifting flakes with wind-modulated horizontal velocity.
**Heritage** -- Parallax-by-layer is a 1980s arcade trick, here applied to atmospheric particles.

#### 54. Parallax Landscape
**Technique** -- Multiple silhouette layers scrolled at different speeds against a procedural sunset gradient.
**Heritage** -- Parallax scrolling was popularized in arcade games like Moon Patrol (1982) and became a hallmark of 16-bit-era platformers.

#### 55. L-System Trees
**Technique** -- String rewriting with branch-stack push/pop, then turtle-graphics interpretation to render the tree; randomized parameters per show.
**Heritage** -- Lindenmayer systems were invented by botanist Aristid Lindenmayer in 1968 to model plant growth.

#### 56. Neon
**Technique** -- Glow halos via inverse-square-distance falloff around shape skeletons, composited over a procedural brick-wall texture, with subtle flicker and a broken-sign effect.
**Heritage** -- Atmospheric neon-tube look from urban-noir film visuals.

### Act 7 -- Retro & Text

#### 57. Lens
**Technique** -- Per-pixel inverse displacement based on a circular spherical-distortion field, sampling a procedural source.
**Heritage** -- Lens warps were demoscene flexes on real-time interpolation and texture mapping.

#### 58. Bump Mapping
**Technique** -- Per-pixel dot product of a procedural height-gradient normal with a moving light direction, faking surface detail without geometry.
**Heritage** -- Introduced by Jim Blinn in 1978.

#### 59. Sine Scroller
**Technique** -- Bitmap-font glyphs blitted with per-character vertical sine displacement and per-row hue shift.
**Heritage** -- Arguably the single most iconic demoscene effect, appearing in virtually every C64 and Amiga demo intro.

#### 60. Oscilloscope
**Technique** -- XY-mode trace into a phosphor-decay accumulation buffer (per-frame multiplicative fade).
**Heritage** -- Recreates the green-phosphor look of analog CRT lab equipment from the 1960s-80s.

#### 61. Pendulum Wave
**Technique** -- A row of pendulums with linearly increasing periods, each at `θ = θ₀ cos(ω_i t)`; collective drift creates traveling waves.
**Heritage** -- Real-world pendulum-wave machines used in physics demonstrations.

#### 62. Spirograph
**Technique** -- Hypotrochoid parametric equations from a point on a circle rolling inside another; integrated into a fading trail.
**Heritage** -- Spirograph toy invented by Denys Fisher in 1965.

#### 63. Flow Field
**Technique** -- Particles advect through a Perlin-noise-derived 2D vector field, leaving colored trails.
**Heritage** -- Popularized by generative artists like Tyler Hobbs in modern creative coding.

#### 64. Pixel Sort
**Technique** -- Per-row pixel-run extraction by brightness threshold, then sorted in place to create digital streak artifacts.
**Heritage** -- Glitch art technique popularized by Kim Asendorf around 2012.

#### 65. Matrix
**Technique** -- Column-based digital rain: each column has its own scroll speed and lead-character brightness, with random katakana glyphs.
**Heritage** -- The "digital rain" of The Matrix (1999), itself inspired by the cascading katakana of Ghost in the Shell.

#### 66. Wobbler
**Technique** -- Bilinear sampling of a procedural source bitmap with two-octave sine displacement on x and y, plus a breathing zoom around screen center.
**Heritage** -- The Amiga "rubber stretch" effect -- distorting a static picture with sinusoidal warps -- was a recurring 1990s flex on real-time addressing tricks.

### Finale

#### 67. Fireworks
**Technique** -- Two-phase particle system: launch (gravity-affected upward) → burst (radial spawn with color-graded fade).
**Heritage** -- Combines projectile physics with radial explosion patterns, a perennial demo finale.

#### 68. Scroller
**Technique** -- Smooth horizontal scroll of a font8x8 bitmap with optional per-glyph vertical sine-wave displacement.
**Heritage** -- The bread-and-butter of every demo since the 1980s, used to deliver the traditional sign-off greetings.

## Building Distribution Packages

A unified build script produces `.deb`, `.rpm`, Arch `.pkg.tar.zst`, and portable `.tar.gz` packages:

```bash
./packaging/build-packages.sh
```

Output lands in `dist/`. The script builds the release binary, strips it, then packages it for each format it can. Formats requiring missing tools are skipped with a message.

**Per-format prerequisites:**

| Format | Requires |
|--------|----------|
| `.deb` | `dpkg-deb` (installed on Debian/Ubuntu by default) |
| `.pkg.tar.zst` | `makepkg` + `bsdtar` (Arch `base-devel` + `libarchive-tools`) |
| `.rpm` | `cargo install cargo-generate-rpm` |
| `.tar.gz` | Nothing beyond `tar` (always available) |

**Install from a package:**

```bash
# Debian / Ubuntu
sudo dpkg -i dist/termdemo_0.1.0_amd64.deb

# Arch
sudo pacman -U dist/termdemo-0.1.0-1-x86_64.pkg.tar.zst

# Fedora / RHEL
sudo rpm -i dist/termdemo-0.1.0-1.x86_64.rpm

# Portable
tar xzf dist/termdemo-0.1.0-linux-x86_64.tar.gz
sudo cp termdemo-0.1.0-linux-x86_64/termdemo /usr/local/bin/
```

## Architecture

Each effect implements a simple trait:

```rust
pub trait Effect {
    fn name(&self) -> &str;
    fn init(&mut self, width: u32, height: u32);
    fn update(&mut self, t: f64, dt: f64, pixels: &mut [(u8, u8, u8)]);
    fn params(&self) -> Vec<ParamDesc>;
    fn set_param(&mut self, name: &str, value: f64);
}
```

The pixel buffer is a flat array of RGB tuples rendered to the terminal using Unicode half-block characters (`\u{2580}`), giving each character cell two vertical pixels. Effects are sequenced with crossfade transitions.

## License

MIT

## Acknowledgments

To the demoscene -- the underground computer art movement that has been pushing creative boundaries on every platform since the 1980s. From C64 cracktros to modern 4KB intros, the scene proved that constraints breed creativity.

Greets to all sceners.
