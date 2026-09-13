# Phase-2 opening review — foreground points approved

The user requested completion of phase 2 and integration of phase 3.
**The user answered yes to retaining unblurred foreground points.** Phase 3 is
now connected in the normal build. The user rejected the separate preview workflow.

The resolved question was whether the preserved nearby stars may remain as
unblurred points after wormhole emergence, with a distant population supplying
the stable background. The user's streak requirement applies before the wormhole.
A zero-foreground-traversal interpretation must not silently become a new spec.
No numerical speed is fixed by the user.

## What was checked

The previous H query included only objects visible in both the 3.95-s and 4-s
views. That sample missed other stars exposed by a different exit route.

The new read-only I query exports all 16,254
existing production star identities and positions, plus the actual opening
camera frame. It does not move objects, alter their visibility or render images.

The reproducible search screens 1,790
bend/orientation candidates that keep the first ring inside the view at 5 s.
Heading is a fixed spatial field integrated into position; speed is still
supplied only by arc length and the same global function. Each candidate uses
a common fixed rotation/translation of opening camera and stars.

The best screened cases still show 15 foreground objects moving faster than
one projected pixel/second at circular entry. One requires a 70-degree turn
over 120,000 km; its fastest retained foreground object moves about 61.3
pixels/second. That turn has **not** been selected or connected live.
Other broad bends still leave moving stars; tighter early bends concentrate
camera rotation and are not accepted merely because they remove more points.

The 1-pixel/s diagnostic and 85-degree/s search bound are engineering screening
choices, **not fixed user specifications**. This is a bounded search, not proof
of impossibility across all paths, speeds or world scales. Rotated out-of-plane
candidates also need complete arc, containment and terrain checks before use.

A new distant population does not make the old foreground population disappear.
The new background's proposed translational parallax is approximately 0.012
centre-screen pixels over the post-exit route; that bound does not apply to
the retained nearby stars.

## What is selected

The current numerical proposal retains the straight incoming asymptote, a
small fixed off-centre excursion within the tunnel, the corrected continuous
planet capture and descending orbits. It does not use one of the large exit
detours. It still has foreground traversal; allowing points does not itself
prove that the resulting background will look sufficiently static.

No star relocation, time fade, spatial mask or renderer switch has been added.
Exact opening raster equality after integration remains an independent gate.
The original executable's hashes remain reference evidence; the normal build
now contains the current changes.

## Next action

No large exit detour is selected. The current build has zero rendered star streaks
after emergence, with the original catalogue intact.

A separate exact-RGB check found old grid pixels in the final approximately
0.02 s before 4 s. Whether those pixels may change with the resized structure
is a new pending question, **not** the foreground-point question already answered.
See [phase-3 integration and resume status](phase3-integration.md).

The query, camera calibration and search are reproducible with:

```sh
make study-opening
```

The complete numerical search report is
`target/phase2-study/opening-review.json`. The current coupled route and camera
results are in [the flight study](phase2-study.md) and [full table](phase2-flight-grid.md).
