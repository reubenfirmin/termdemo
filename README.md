# termdemo

A tiny, dependency-free procedural mesh landscape demo rendered through the Kitty
graphics protocol. The executable is freestanding Rust: `no_std`, `no_main`,
no allocator, no libc, and no external crates.

## Current status

[PLAN.md](PLAN.md) is the active design specification. The new single-function
flight and Earth-sized world are connected in the normal build, not yet
accepted. The original foreground stars remain fixed and become unblurred
points after the wormhole, as approved.

[Integration status and measurements](docs/phase3-integration.md) records two
failing gates: the resized grid changes a few pixels in the final approximately
0.02 s of the protected opening, and some low-flight frames exceed the time
budget. The camera and original star catalogue are byte-identical through 4 s.
The grid-pixel permission question remains open; no reference images/hashes
were regenerated.

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
```

The live audit checks actual camera derivatives at 1,920 Hz, exact opening
camera/star preservation, protected RGB, independent route agreement, world
dimensions, tunnel containment, terrain clearance, object topology, clipping,
rendered grid/star visibility, frame time and playback controls. Results are
in `target/phase3-audit/result.json`. It currently returns failure for the
opening-grid conflict and frame time; numerical motion checks do not waive these.

The ten harness unit tests use fixtures and negative witnesses. The twelve
offline study tests check the independent math. Neither suite claims that
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
