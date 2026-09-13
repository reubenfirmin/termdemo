# termdemo

A tiny, dependency-free procedural mesh landscape demo rendered through the Kitty
graphics protocol. The executable is freestanding Rust: `no_std`, `no_main`,
no allocator, no libc, and no external crates.

## Current status

[PLAN.md](PLAN.md) is the active design specification. The new single-function
flight and Earth-sized world are connected in the normal build. Phase 3's
implementation and automated checks are complete; visual acceptance of the
detailed descent remains later work. The original foreground stars remain fixed
and become unblurred points after the wormhole, as approved.

[Integration status and measurements](docs/phase3-integration.md) records all
25 passing gates (25.68-ms worst audit frame; target 33.33 ms). The complete
2,311-frame render sequence peaked at 27.78 ms. The camera and original star
catalogue remain exact through 4 s. The limited late-opening grid revision is recorded separately;
the original references and all pixels outside its 16×16 rectangle are protected.

Use the normal workflow: `make build` / `make run`. There is no separate preview
target; the executable is `target/x86_64-unknown-linux-gnu/release/termdemo`.
Known test failures remain reported rather than keeping the normal executable
on an older version. Playback pauses at 77 s; surface manual flight is not
implemented yet.

The development order is:

1. Behavior-preserving code hygiene and trustworthy measurements.
2. Complete the coupled world dimensions, route and speed calculation.
3. Connect the single distance-driven flight with its required basic world dimensions.
4. Develop fractal coasts, relief, drainage, settlements and continuous refinement.
5. Add surface manual control and finish geometric, performance and live visual verification.

The approved opening remains protected. The wormhole is around 4–5 s; rings
are already distantly visible at emergence. The whole 7.6–23 s interval is the
assembling circular structure, evolving into the square near its end. There
is no new post-wormhole acceleration/streak act. The accepted numerical
baseline reaches orbit 3 entry around 41.419 s, then requires rapid, smooth
altitude/speed loss without completing that lap. Revised surface arrival
is proposed at 77 s, via airline height at 57 s, without completing orbit 3.
There are no fixed numerical speed specifications: speeds are fitted outputs.
These timings are now reproduced by the actual camera; they are not visual approval.

The planet specification uses distinct, connected fractal/procedural systems:
coasts, inland-high/coastal-low continental relief, branching mountain ranges,
terrain-driven rivers and basin lakes, and terrain-aware city placement/layouts.
All refine the same fixed geometry. See [the planetary work packages](PLAN.md#later-phase-planetary-geography).
No imagery, tiled landscape replacement, independent descent clock or renderer switch.

Earlier descriptions and acceptance claims are [archived](docs/history/README-before-phase1.md),
not current instructions. The plan's older checkpoints are [archived separately](docs/history/flight-plan.md).

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
checkpoints. Space pauses/resumes the exact current frame and its timestamp;
resuming does not skip any paused time. Seeking while paused updates the view
without resuming playback. Left Arrow, `[`, `,`, `<`, or
`b` move back two seconds; Right Arrow, `]`, `.`, `>`, or `f` move forward two seconds. The
upper-left `SSS.t` counter identifies transition timing to a tenth of a second.
The program uses the alternate screen and restores the original terminal mode
on normal exit.

`make audit-controls` checks pause/resume timing, held-frame stability, paused
seeking and terminal cleanup without running image-baseline checks.

Manual flight controls are not implemented yet; Space and the seek keys control
playback, not the spacecraft.

When stdout is not a terminal, the executable renders and encodes one frame and
then exits. This provides a headless smoke test:

```sh
target/x86_64-unknown-linux-gnu/release/termdemo </dev/null >/dev/null
```

## Verification

```sh
make check
make audit-harness
make audit-study
make audit-phase3-live
make profile-phase3-render
```

The live audit checks actual camera derivatives at 1,920 Hz, exact opening
camera/star preservation, protected RGB, independent route agreement, world
dimensions, tunnel containment, terrain clearance, object topology, clipping,
rendered grid/star/Sun visibility, fixed-plane passage cadence, normal/audit RGB
parity, serial/parallel RGB and depth, worker-failure recovery, frame time and
playback controls. Results are in `target/phase3-audit/result.json`; all 25 gates
currently pass. Timing remains host-dependent, not a universal real-time guarantee.
The stage profiler records elapsed frame time and total rendering CPU costs in
`target/phase3-audit/profile.json`, using the existing audit instrumentation.
It does not create a separate app build or change compilation settings.

The renderer uses up to three persistent Linux worker children plus the parent.
They claim eight-row tiles from a shared queue and evaluate the same frame and
pixel samples. Completed RGB/depth tiles occupy disjoint shared rows. Ordered
grid commands are binned once per frame, not projected by every worker.
Serial fallback preserves output if workers cannot
be started or a channel fails. Normal exit closes and reaps all owned children.
Wall-time measurements include dispatch and assembly; CPU costs sum all workers,
not just the parent. No reduction in resolution, spokes or terrain detail.

To profile successive moving frames instead of repeated frozen views:

```sh
node scripts/profile-phase3-render.cjs --sequence       # 0–77 s, sampled at 30 Hz
node scripts/profile-phase3-render.cjs --sequence 60 73
node scripts/profile-phase3-render.cjs --serial 65 70   # same kernel, serial baseline
make profile-phase3-playback                           # full normal-program PTY run
```

The sequence profiler reports every budget exceedance and observed host metadata.
The playback profiler measures completed protocol transmissions and the existing
timestamp counter through 77 s, including terminal cleanup. Neither claims actual
Kitty display/compositor FPS or visual approval. Reports remain under
`target/phase3-audit/`; these are diagnostics, not separate application builds.

The fourteen harness unit tests use fixtures and negative witnesses, including
the playback counter decoder. The twelve offline study tests check the
independent math. Neither suite claims that
the current source is an unchanged phase-1 extraction.

Normal frame comparisons hash RGB in memory; no image baseline is regenerated.
The old `audit-extraction`, `audit-phase0` through `audit-phase7` and the
withdrawn `audit-speed-function` refer to archived implementations, not current
phase acceptance. Their old references are preserved, not updated to pass.
The legacy audit protocol now explicitly rejects obsolete selectors.

## Size policy

Size optimization is deliberately deferred during visual development. The
broader target remains a polished visual executable below 32 KiB, leaving half
of a 64 KiB envelope for sound and additional scenes. Runtime memory is
generated procedurally and does not contribute to the file size.
