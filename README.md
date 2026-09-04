# termdemo

A tiny, dependency-free procedural mesh landscape demo rendered through the Kitty
graphics protocol. The executable is freestanding Rust: `no_std`, `no_main`,
no allocator, no libc, and no external crates.

The demo is a continuous six-act sequence:

1. Eleven uninterrupted seconds of forward flight through a rectangular-frustum
   star field with stable stellar temperatures, depth bloom, rare attached
   sparkles, and an irregular galactic dust band before organization begins.
   The first four seconds preserve a sparse, pure point-star flow. New stars
   join only at the origin, the population fills gradually, and a minimum
   radial velocity carries every star completely beyond a viewport edge before
   it can recycle. Each star then has its own fixed streak-onset time, so
   isolated streaks appear first and their population increases continuously
   before the entire field stretches
2. The same 3,625 finite star streaks becoming the conical mesh without a
   renderer swap. Every element belongs to the same continuous outward stream.
   A persistent set of ordered radial spokes gradually strengthens beneath the
   still-random streaks. Complete threshold rings expand across those spokes
   from the exact origin without changing the shading on either side. Over
   more than five seconds, their
   initially wide emission gaps contract geometrically into the regular grid
   cadence. The first regular-cadence ring is also the boundary of the clean
   black cone: it carries the dark field outward with the grid's exact depth
   and projection law while new regular rings remain visible inside it, joined
   to the established outer radials. The old streak packets continue past the
   camera ahead of that front. After the field fills, off-screen rings recycle
   through the same emitter. The projected cone
   extends well
   beyond every viewport edge rather than terminating in a fitted ellipse. No
   replacement edges, hidden side grids, or late subdivision lines are
   introduced
3. The connected radial mesh squaring its circular cross-section into a broad
   rectangular tunnel while its left and right faces push far beyond the
   viewport. Each recycled ring inherits the current reshaping at its origin
   birth and carries that shape toward the camera, so the change propagates
   forward without replacing geometry. The most distant bands fade out before
   reaching the origin, where their density would resolve only as blur. Once
   reshaping begins, stable full-spectrum lanes are carried outward by newly
   emitted mesh segments rather than changing the whole field at once. Color
   emission reaches full strength two seconds before the tunnel becomes
   horizontal, giving its last pale segments time to leave. Bright corner
   rails and restrained bloom separate the ceiling, floor, and both side
   grids. Moving light pools shade connected regions of all four faces once
   the tunnel squares up, while a fixed world-space sun establishes the
   lighting direction used by the planet and terrain and naturally enters the
   camera view again over the final valley. Rings recycle one at a time,
   maintaining forward parallax down the grid's center. From the clean-cone
   front to the planet reveal, accumulated grid travel eases smoothly down to
   a slow approach
4. The same grid bending continuously into a broad gravity well on its bottom
   face. The visible well arrives at the distance-fog boundary before its
   textured, per-pixel ray-sphere planet is revealed at that established grid
   anchor. One uninterrupted seventy-two-second approach and descending spiral
   then carries the moving surface beneath the camera through five
   same-direction revolutions. Its position and velocity remain continuous
   across every timing boundary, total forward speed never reaches zero, and the boxed grid revolves above
   it and remains visible through most of the descent. The sphere grows into
   the lower horizon; the grid continues behind it into entry and disappears
   only as the expanding surface physically occludes it. Terminal pixel geometry keeps the sphere
   physically round at every window shape. Moving diffuse and
   water-dependent specular light, clouds, ocean, continents,
   and a narrow cyan/green/violet aurora resolve across its limb
5. A single uninterrupted atmospheric passage: turbulent edge-bound entry plasma builds
   on top of the still-visible moving globe. Its spherical terrain coordinates
   continuously unwrap into the landing map after the globe covers the entire
   viewport, retaining the same river, coast, relief, lighting, and motion on
   both sides of the tangent projection. That shared height field then moves
   from roughly 1,000-mile continent scale through 100-mile regional scale
6. The same per-pixel terrain camera keeps descending and pitching from the
   regional overhead view to a valley seen from above. Increasingly forward-looking rays
   allow distant mountain ranges to rise across the geometric horizon before
   their walls enclose the camera at roughly one-mile scale. Plasma remains a
   translucent atmospheric overlay and converges to zero without replacing
   the terrain beneath it; terrain fog and world-anchored clouds remain part of
   the same continuously sampled view

The final scene flies at 3.45 terrain-map units per second along a carved,
periodic valley. Its continuous world-space tangent tracks the valley center
while altitude and look angle settle smoothly from the orbital path. A single
master world path supplies orbital phase, radial approach,
grid orientation, surface time, atmospheric progress, and nonzero ground
velocity from first sighting onward. Terrain-space lighting and refined height-field intersections keep
the mountain shading stable as the surface moves. World-anchored crags,
strata, and vegetation variation add motion cues to the valley walls while leaving the floor quieter.
Frames update Kitty's persistent root animation buffer so they do not create
and delete screen placements.

The geometric sequence uses 73 angular lanes and 25 live depth bands. Its
wireframe is rasterized from subpixel endpoints with antialiased hot cores,
soft bloom, rotating directional light, specular highlights, and traveling
depth glints. The earlier ship deceleration uses a front-loaded smooth curve,
while distance-based recycling keeps the grid spacing uniform at every speed.

## Requirements

- x86-64 Linux
- Kitty 0.43.1 or newer
- a true-color terminal window large enough to enjoy the view

Kitty versions through 0.46.2 have a security issue in the PNG graphics path.
This demo only transmits raw RGB24 data and does not exercise that path, but the
terminal version remains vulnerable to malicious PNG protocol input elsewhere.

## Build and run

```sh
make size
make run
```

`make small` also creates `target/x86_64-unknown-linux-gnu/release/termdemo-small`.
It removes section metadata that the Linux loader does not need. There is no
hard size gate while the renderer and art direction are taking shape.

Run it directly in Kitty rather than through tmux or another multiplexer.
Press Escape, `q`, or Ctrl-C to leave. Keys `1` through `6` jump to timeline
checkpoints. Space skips forward five seconds. Left Arrow, `[`, `,`, `<`, or
`b` move back two seconds; Right Arrow, `]`, `.`, `>`, or `f` move forward two seconds. The
upper-left `SSS.t` counter identifies transition timing to a tenth of a second.
The program uses the alternate screen and restores the original terminal mode
on normal exit.

The uninterrupted sequence reaches the low valley flight after 137 seconds.

When stdout is not a terminal, the executable renders and encodes one frame and
then exits. This provides a headless smoke test:

```sh
target/x86_64-unknown-linux-gnu/release/termdemo </dev/null >/dev/null
```

## Size policy

Size optimization is deliberately deferred during visual development. The
broader target remains a polished visual executable below 32 KiB, leaving half
of a 64 KiB envelope for sound and additional scenes. Runtime memory is
generated procedurally and does not contribute to the file size.
