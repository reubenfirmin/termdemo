# termdemo

A tiny, dependency-free procedural mesh landscape demo rendered through the Kitty
graphics protocol. The executable is freestanding Rust: `no_std`, `no_main`,
no allocator, no libc, and no external crates.

The intended demo is a continuous six-act sequence. Acts 1 through 3 and the
planet approach through orbit four are accepted. A candidate orbit-to-valley
trajectory is now connected to that same camera and awaits visual approval;
the depth-tested spherical surface and continuous geometric refinement are
connected, while atmospheric art remains unfinished. The binding
implementation plan is in [PLAN.md](PLAN.md).

The sequence is:

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
4. The same grid bends into a gravity well on its bottom face. A wireframe
   sphere is introduced at that grid anchor at 27 seconds, and the shared
   projective camera begins bending the approach into orbit at 28 seconds. The
   grid section, well, and planet share translation and projection. The planet
   remains physically round at every terminal aspect ratio. Demo time and path
   time remain 1:1: incoming speed falls while velocity bends from radial
   approach into tangential travel, and the tightening radius supplies the
   faster angular turnover. The path reaches the fourth revolution at
   approximately 51.959 seconds. For the
   working physical model of a 27,000-mile-diameter planet, that point is
   exactly 100 miles above the surface. The candidate continuation completes
   almost four further same-direction revolutions while braking and descending
   continuously, reaches the destination region from above at 94 seconds, and
   uses the remaining motion to fly along the valley. It reaches continent
   scale by 78 seconds and 20 metres at 106 seconds, then continues low flight
   through 137 seconds.
5. **Planned, not yet implemented on the accepted camera:** the fourth orbit
   continues directly into atmospheric descent. Plasma and atmospheric
   scattering accumulate over the already visible grid and globe according to
   density and velocity. No renderer, camera, coordinate system, mask, cut, or
   crossfade may replace the orbital view.
6. The globe and terrain are one depth-tested, canonically displaced spherical
   surface; rear grid fragments are occluded while genuinely nearer grid
   fragments remain visible. Nested parent geometry refines continuously by
   projected footprint from continent to regional landscape, mountain range,
   valley-from-above, and finally valley flight at 20 metres above displaced
   terrain. Curvature becomes locally imperceptible through scale alone; the
   planet never turns into a separate plane.

Legacy terrain routines remain in the source, but the generated height source
now also displaces the canonical spherical surface. The legacy camera is confined to the protected segment
before unified-camera entry at 28 seconds. After entry, the grid, wire planet,
sun, sky-density input, and entry-sheath input all use `FlowCamera`; remaining
terrain formulas will be removed or migrated according to [PLAN.md](PLAN.md).

Frames update Kitty's persistent root animation buffer so they do not create
and delete screen placements.

The geometric sequence uses 73 angular lanes and 25 live depth bands. Its
wireframe is rasterized from subpixel endpoints with antialiased hot cores,
soft bloom, rotating directional light, specular highlights, and traveling
depth glints. The earlier ship deceleration uses a front-loaded smooth curve,
while distance-based recycling keeps the grid spacing uniform before orbit
capture. During orbit, the nearby grid section shares the planet's co-moving
world frame so its floor, ceiling, and walls remain present around the camera.

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

The target sequence reaches low valley flight after 137 seconds. The currently
accepted visual implementation ends at orbit four; the unified candidate path
now reaches the 20-metre endpoint, but is not accepted until its motion passes
user visual review. Legacy terrain output is not considered completion of that
journey.

When stdout is not a terminal, the executable renders and encodes one frame and
then exits. This provides a headless smoke test:

```sh
target/x86_64-unknown-linux-gnu/release/termdemo </dev/null >/dev/null
```

The protected Phase 0 regression gate and the physical-scale/precision gate are
also headless:

```sh
make audit-phase0
make audit-phase1
make audit-phase2
make audit-phase3
make audit-phase4
make audit-phase5
```

It verifies ten approved pre-planet frame hashes, orbit-entry and fourth-orbit
invariants, terminal aspect compensation, seeking controls, Escape behavior,
and terminal restoration. The Phase 1 gate additionally verifies physical-unit
round trips and stable camera-relative recovery of the 20-metre endpoint. The
Phase 2 gate checks boundary derivatives, every altitude control, monotonic
descent, orbital direction, camera/path alignment, endpoint motion, and valley
arrival. The Phase 3 gate verifies the post-entry camera quarantine, fixed
world-space sun, unified atmospheric inputs, former-boundary continuity, and
deterministic seeking. The Phase 4 gate checks stable spherical terrain
placement, shared depth rejection, and a 20-metre endpoint above the displaced
surface. The Phase 5 gate checks ordered parent/child refinement, exact height
reconstruction, attached landmarks, subpixel geomorphing, and the actual
destination valley’s visibility before entry.
`make audit-phase0-baseline` deliberately regenerates the frame hashes and must
only be used after explicit visual approval.

## Size policy

Size optimization is deliberately deferred during visual development. The
broader target remains a polished visual executable below 32 KiB, leaving half
of a 64 KiB envelope for sound and additional scenes. Runtime memory is
generated procedurally and does not contribute to the file size.
