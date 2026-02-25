# termdemo

A love letter to the demoscene, running entirely in your terminal. 63 real-time visual effects rendered at 60fps using Unicode half-block characters for square pixel output. No GPU, no dependencies beyond a terminal emulator.

## Quick Start

```bash
cargo run --release          # autoplay: sit back and watch
cargo run --release -- -i    # interactive: browse effects manually
```

## Controls

| Key | Action |
|-----|--------|
| `q` / `Esc` | Quit |
| `Space` | Pause / resume |
| `Tab` | Toggle autoplay / interactive mode |
| `n` / `Right` | Next effect |
| `p` / `Left` | Previous effect |
| `h` | Toggle HUD overlay |
| `Up` / `Down` | Adjust current effect parameter |
| `1`-`9` | Jump to effect 1-9 |

## Requirements

- Rust 1.56+ (2021 edition)
- A terminal with true-color (24-bit) support
- Recommended: 80x24 minimum, larger terminals look better

## The Effects

### Act 1 -- Classic Patterns

| # | Effect | History |
|---|--------|---------|
| 1 | **Plasma** | The quintessential demoscene effect. Overlapping sine waves in color space, first popularized on the Amiga in the late 1980s. Every demo group had their own variant. |
| 2 | **Moire** | Overlapping concentric circle patterns that create shimmering interference fringes. Named after the French textile weaving technique, moire patterns became a staple of early computer graphics. |
| 3 | **Kaleidoscope** | Mirrors a pattern across multiple axes of symmetry, emulating the Victorian-era optical toy invented by David Brewster in 1816. A natural fit for real-time graphics. |
| 4 | **Shadebobs** | Additive light blobs that leave glowing trails as they orbit. A signature effect of Amiga demos in the early 1990s, exploiting the hardware's blitter for fast screen compositing. |
| 5 | **Copper Bars** | Horizontal color gradient bars, named after the Amiga's Copper coprocessor which could change palette registers mid-scanline. A defining visual of the Amiga demoscene. |
| 6 | **Raster Bars** | Close cousin of copper bars -- horizontal bars with per-scanline color manipulation. Originally a C64 technique using raster interrupts to change border and background colors. |
| 7 | **Copper Flag** | A waving flag rendered with copper bar-style horizontal stripes and sine-wave distortion. Combines the copper bar palette trick with cloth-like wave animation. |
| 8 | **Kefrens Bars** | Vertical bars with per-scanline horizontal displacement, creating a weaving curtain. Named after the legendary Danish demo group Kefrens, whose Amiga demos made this iconic. |
| 9 | **Truchet** | Randomly oriented quarter-circle tiles forming flowing maze-like patterns. Based on the tilework of Sebastien Truchet (1704), rediscovered by Cyril Stanley Smith in 1987. |
| 10 | **Interference** | Two-source wave interference creating bright and dark fringes. Simulates Thomas Young's 1801 double-slit experiment that proved the wave nature of light. |

### Act 2 -- Heat & Motion

| # | Effect | History |
|---|--------|---------|
| 11 | **Fire** | The classic real-time fire algorithm: heat rises from the bottom, diffuses, and cools. Popularized by demos on the PC in the early 1990s, with variants appearing on every platform. |
| 12 | **Twister** | A rotating rectangular bar with four colored faces, using sine-based edge projection. A signature effect of 1990s Amiga and PC demos, requiring only 1D math per scanline. |
| 13 | **Tunnel** | Texture-mapped infinite tunnel using polar coordinate lookup tables. First appeared in PC demos around 1993 and became one of the most recognizable demoscene effects. |
| 14 | **Dot Tunnel** | Rings of dots receding into the screen, creating a tunnel from discrete points. A lighter variant of the solid tunnel popular on 8-bit and 16-bit platforms where fill rate was limited. |
| 15 | **Rotozoom** | A rotating and zooming texture, computed by inverse-mapping each screen pixel through a 2D rotation matrix. A staple of the Amiga and Atari ST demo scenes. |
| 16 | **Lightning** | Procedural branching lightning bolts with flash illumination. Uses recursive midpoint displacement to generate the jagged bolt path, a technique from fractal terrain generation. |
| 17 | **Lava Lamp** | Soft blobby shapes rising and falling with organic deformation, emulating the 1963 invention by Edward Craven Walker. Implemented using a metaball field with warm color mapping. |

### Act 3 -- 3D Geometry

| # | Effect | History |
|---|--------|---------|
| 18 | **Starfield** | The classic warp-speed starfield: stars flying toward the viewer from a central vanishing point. One of the very first demo effects, dating back to the C64 era. |
| 19 | **Galaxy** | Spiral galaxy particle system with arms following logarithmic spiral equations. Inspired by density wave theory explaining real galactic structure. |
| 20 | **Dot Sphere** | Points distributed on a rotating sphere using the Fibonacci spiral method. A descendant of the dot-based 3D effects popular on 8-bit platforms. |
| 21 | **Boing Ball** | The iconic 1984 Amiga Boing Ball demo: a red-and-white checkered sphere bouncing in a purple grid room. The effect that introduced the Amiga at CES and became its unofficial mascot. |
| 22 | **Filled Vector** | Flat-shaded rotating icosahedron with painter's algorithm depth sorting. Represents the leap from wireframe to solid 3D that happened in demos around 1990-1992. |
| 23 | **Morph** | Point cloud smoothly interpolating between shapes (sphere, cube, torus). 3D morphing became a demo staple after the technique appeared in films like Terminator 2 (1991). |
| 24 | **Glenz** | Transparent overlapping 3D objects with additive color blending. Named after the "Glenz vector" style popularized by groups like Future Crew in their landmark PC demos. |
| 25 | **Lissajous 3D** | 3D Lissajous curves -- parametric paths from orthogonal sine waves. Named after Jules Antoine Lissajous who studied them in 1857 using tuning forks and mirrors. |
| 26 | **Torus Knot** | A curve that winds around a torus surface, forming beautiful knot patterns. Torus knots are studied in mathematical knot theory and became popular in 2000s demos and screensavers. |
| 27 | **Wireframe** | Classic wireframe 3D object rotation with hidden-line removal. The original mode of real-time 3D graphics, dating to Ivan Sutherland's Sketchpad (1963). |
| 28 | **Cube Field** | Flying through an infinite field of flat-shaded cubes. Inspired by the Flash game "Cubefield" (2006) and the endless runner genre, adapted as a demoscene fly-through. |
| 29 | **Wolfenstein** | Raycasting pseudo-3D engine in the style of Wolfenstein 3D (1992). John Carmack's DDA raycasting algorithm rendered a full 3D-looking world from a 2D map, revolutionizing games. |
| 30 | **Raymarcher** | Sphere-tracing signed distance fields to render smooth organic 3D shapes. Pioneered by demosceners like iq (Inigo Quilez) for creating stunning 4KB intros. |
| 31 | **Terrain** | Heightmap terrain flyover using column-based raycasting, inspired by the Comanche engine (NovaLogic, 1992) which rendered voxel landscapes in real-time on 386 PCs. |
| 32 | **Voxel Landscape** | Voxel terrain rendering in the style of Comanche. Each column of pixels is cast into the world to sample a height and color map, creating a convincing 3D landscape. |

### Act 4 -- Fractals

| # | Effect | History |
|---|--------|---------|
| 33 | **Mandelbrot** | The iconic Mandelbrot set, discovered by Benoit Mandelbrot in 1980. Iterating z = z^2 + c in the complex plane reveals infinite self-similar detail at every zoom level. |
| 34 | **Julia** | Julia sets -- close relatives of the Mandelbrot set, explored by Gaston Julia in the 1910s. Each point in the Mandelbrot set corresponds to a unique Julia set. |
| 35 | **Fractal Zoom** | Infinite smooth zoom into the Mandelbrot set's Seahorse Valley, revealing layer after layer of self-similar spiral structures. A mesmerizing demonstration of fractal depth. |
| 36 | **Sierpinski** | The Sierpinski triangle built via the chaos game algorithm -- randomly jumping halfway toward triangle vertices. Discovered by Waclaw Sierpinski in 1915, the chaos game variant by Michael Barnsley. |

### Act 5 -- Simulations

| # | Effect | History |
|---|--------|---------|
| 37 | **Metaballs** | Implicit surface blobs that merge smoothly when close. Invented by Jim Blinn in 1982 as "blobby molecules," they became a signature effect of 1990s demos and the 2000s web (Flash). |
| 38 | **Voronoi** | Voronoi diagram -- partitioning space by nearest seed point. Named after Georgy Voronoy (1908), these diagrams appear everywhere from cell biology to airport coverage maps. |
| 39 | **Reaction-Diffusion** | Gray-Scott model: two chemicals diffuse and react, spontaneously forming spots, stripes, and labyrinthine patterns. Alan Turing proposed reaction-diffusion as the basis of biological morphogenesis in 1952. |
| 40 | **Fluid Simulation** | Jos Stam's stable fluids algorithm (1999): diffuse, advect, project. A simplified Navier-Stokes solver that made real-time fluid simulation practical for games and demos. |
| 41 | **Cloth Simulation** | Verlet integration with distance constraints, the method popularized by Thomas Jakobsen (2001). Pin points, gravity, and wind forces create natural fabric motion. |
| 42 | **Water** | 2D ripple simulation using a height field. Each cell averages its neighbors and dampens, creating expanding concentric wave patterns when disturbed. A classic 1990s DOS effect. |
| 43 | **Fountain** | Particle system fountain with gravity, emitting a continuous stream of particles that arc and fall. Particle systems were formalized by Bill Reeves at Lucasfilm for Star Trek II (1982). |
| 44 | **Boids** | Craig Reynolds' 1986 flocking algorithm: separation, alignment, and cohesion rules produce emergent bird-like swarm behavior from simple local interactions. |
| 45 | **Cellular Automata** | Brian's Brain -- a 3-state cellular automaton (off/on/dying) that produces chaotic moving patterns with gliders and oscillators. A variation on the cellular automata framework pioneered by John von Neumann. |
| 46 | **Game of Life** | John Conway's 1970 cellular automaton: cells live or die by simple neighbor-count rules, yet produce gliders, guns, and even Turing-complete computation. |

### Act 6 -- Natural & Atmospheric

| # | Effect | History |
|---|--------|---------|
| 47 | **Aurora Borealis** | Layered curtains of light simulating the northern lights. Real aurorae are caused by solar wind particles exciting atmospheric gases; here we use sine-wave curtains with additive blending. |
| 48 | **Rain** | Heavy rain with parallax depth layers, splash particles, and lightning flashes. Combines multiple classic techniques: particle systems, layered scrolling, and procedural flash events. |
| 49 | **Snowfall** | Parallax snowflakes drifting and accumulating. Three depth layers create a convincing sense of 3D space, with wind affecting drift patterns. |
| 50 | **Parallax Landscape** | Layered mountain silhouettes scrolling at different speeds against a sunset sky. Parallax scrolling was pioneered in arcade games like Moon Patrol (1982) and became a hallmark of 16-bit era platformers. |
| 51 | **L-System Trees** | Fractal trees generated by Lindenmayer systems, the string-rewriting formalism invented by botanist Aristid Lindenmayer in 1968 to model plant growth. Turtle graphics interpretation produces natural branching. |
| 52 | **Neon** | Glowing neon sign shapes with inverse-square-distance glow halos on a brick wall. Emulates the warm atmospheric glow of real neon tubes, with subtle flicker and a broken-sign effect. |

### Act 7 -- Retro & Text

| # | Effect | History |
|---|--------|---------|
| 53 | **Lens** | A magnifying lens distortion that warps the underlying texture. Lens effects appeared in demos as a way to show off real-time texture mapping and interpolation. |
| 54 | **Bump Mapping** | Per-pixel lighting on a height map to simulate surface detail. Introduced by Jim Blinn in 1978, bump mapping gives the illusion of geometry without additional polygons. |
| 55 | **Sine Scroller** | Large text scrolling along a sine wave path with rainbow coloring. The sine scroller is arguably the single most iconic demoscene effect, appearing in virtually every C64 and Amiga demo. |
| 56 | **Oscilloscope** | XY-mode Lissajous figures with phosphor persistence, emulating an analog CRT oscilloscope. The green phosphor glow and slow decay recreate the look of lab equipment from the 1960s-80s. |
| 57 | **Pendulum Wave** | A row of pendulums with slightly different periods that drift in and out of sync, creating mesmerizing wave patterns. Based on real-world pendulum wave machines used in physics demonstrations. |
| 58 | **Spirograph** | Hypotrochoid curves tracing themselves with color trails, emulating the Spirograph toy invented by Denys Fisher in 1965. Mathematical curves from rolling circles within circles. |
| 59 | **Flow Field** | Particles following a Perlin-like noise vector field, leaving colored trails. Flow field art was popularized by generative artists like Tyler Hobbs and became iconic in modern creative coding. |
| 60 | **Pixel Sort** | Glitch art technique: sorting pixel runs by brightness to create digital streak artifacts. Originated in the creative coding community around 2012, popularized by artist Kim Asendorf. |
| 61 | **Matrix** | The "digital rain" from The Matrix (1999), itself inspired by the cascading katakana of Ghost in the Shell. Green characters falling in columns with variable speed and brightness. |

### Finale

| # | Effect | History |
|---|--------|---------|
| 62 | **Fireworks** | Particle-based fireworks with launch, burst, and gravity-affected trails. Combines projectile physics with radial explosion patterns and color fading. |
| 63 | **Scroller** | Horizontal scrolling text -- the bread and butter of every demo since the 1980s. Used here to deliver greetings, the traditional demoscene sign-off. |

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

## Building

```bash
cargo build --release    # optimized build (recommended for smooth playback)
cargo run --release      # build and run
```

## License

MIT

## Acknowledgments

To the demoscene -- the underground computer art movement that has been pushing creative boundaries on every platform since the 1980s. From C64 cracktros to modern 4KB intros, the scene proved that constraints breed creativity.

Greets to all sceners.
