# Front-loaded flight grid — numerical candidate, 2026-09-12

NOT IMPLEMENTED and NOT a fully fitted flight. The user accepted the table's
front-loaded journey as the planning baseline, with one explicit exception:
**do not complete orbit 3. At its beginning, rapidly and smoothly reduce altitude
and total speed together toward continental flight and the canyons.**

Rows through orbit 3 entry (41.419 s) are the approximate accepted baseline,
not proof of a fitted incoming route, terrain clearance or live implementation.
Rows after that event, including the 148.657 s third-lap/aircraft endpoint, are
**superseded diagnostic output**, retained below only as the original calculation.
They are not the planned continuation. The exact function below was fitted
using third-lap completion, so its coefficients and the geometric descent must
be refitted together as ONE global speed law and ONE route, preserving the
accepted early pacing and reporting any changes. Truncating or relabelling
this table does not implement the new descent; no separate late brake is allowed.

The old 40-second aircraft deadline does not fit the accepted orbit ordering;
new aircraft and surface arrival times remain unresolved. Neither a 58-second
surface time nor the old 148.657-second arrival is a solved revised endpoint.

Planetary feature fidelity is part of flight acceptance: persistent procedural
continental features must resolve into mountains, valleys, cities and canyons
through the same surface geometry, without imagery, tiled repetition, popping
or a renderer change. The simultaneous descent/braking must be readable in
uninterrupted playback, with pause available for inspecting those same landmarks.

## Assumptions and scope

- Earth radius: 6,371,000 m. The production planet has not been resized.
- Baseline initial speed at 5 seconds: 67323240.54277262 m/s. Preserve the accepted
  early pacing when refitting; this is not an installed or independently locked coefficient.
- Initial inward approach is not yet fitted. Pre-orbit event times are targets.
- Orbit-only witness begins at 28 seconds, after the space approach, at a
  reference-sphere altitude of 300 km. This is a study choice, not a prescribed
  incoming flight angle or fixed altitude profile for the final route.
- The original witness decreases reference altitude continuously with angular
  travel to 9,144 m after three revolutions. That endpoint constraint is now
  superseded. These are NOT terrain-relative clearance measurements.
- A "far-side pass" in this table means half a revolution after that orbit's
  start. It does not claim a separately validated crossing of the physical grid.
- The 100-km marker is a reference atmosphere boundary, not a rendered visibility proof.
- Near-surface/canyon arrival and manual takeover remain unfitted. They cannot
  honestly be labelled as occurring at 58 seconds.

## Original analytic speed function — refit required

Metres and seconds throughout; for t >= 4:

```
u = t - 4
w = 0.25
g(u) = u - 1.5*w + 2*w*exp(-u/w) - 0.5*w*exp(-2*u/w)

k = 0.08269318894118219
A = 67323107.54277262
g(1) = 0.6341158866158793
v(t) = 133 + A*exp(-k*(g(t-4) - g(1)))
s(t) = integral(v(tau), tau=5..t)
```

Since g'(u)=(1-exp(-u/w))^2, speed decreases monotonically. The incoming
speed's first two time derivatives vanish at t=4; full opening projection
parity still needs verification. Maximum absolute deceleration occurs at
5.1505s, not at the tail. Its value is 5390377.6 m/s²:
this is fantastical cinematic motion, not a passenger-safe physical claim.
Worst one-second speed loss after 5s is 7.936623%.
The short onset blend does not introduce a new controller or scene boundary.

## Original orbit-only geometric witness — third-lap endpoint superseded

```
theta in [0, 6*pi]
r(theta) = 6371000 + 9144 + (300000-9144)*(1-theta/(6*pi))^3
arc(theta) = integral(sqrt(r(q)^2 + r'(q)^2), q=0..theta)
theta(t) = inverse_arc(integral(v(tau), tau=28..t))
```

Thus radial and tangential motion together use the same TOTAL speed. The
amplitude and k were solved numerically so orbit 1 completes at 33s and orbit 3
completes where v reaches 600 m/s. That latter time is 148.656797s,
NOT 40s. The full incoming camera frame, containment, surface displacement and
subsequent descent have not been fitted or verified.

Orbit 1 arc: 41188.045543 km; total three-orbit arc:
121634.123352 km. Orbit durations are
5.000000s, 8.418966s,
and 107.237831s. This long third orbit is a failure
against the requested pacing, not an approved timing revision.

The next 18 seconds after aircraft-speed arrival provide only
6.766677 km of total travel, below the required 9.124-km
vertical drop. 178.420061s is only an idealized
all-vertical distance bound, NOT a proposed surface-arrival time.

## Original per-second grid, with fractional milestone rows

All integer seconds from 5s through the original aircraft-speed event are included.
Only the rows through orbit 3 entry form the accepted approximate baseline;
the subsequent rows are superseded, not a revised flight proposal.
Percent remaining is relative to v(5); one-second loss compares v(t) with v(t-1),
not with the preceding table row. Values are rounded for display.

| Time, s | Event | Total m/s | % of exit | Lost in last 1s | Reference altitude, m | Orbits completed | Distance since 5s, km |
|---:|---|---:|---:|---:|---:|---:|---:|
| 5 | Wormhole exit (target) | 67,323,241 | 100.00% | — | Not fitted | — | 0.000 |
| 6 | — | 62,025,925 | 92.13% | 7.87% | Not fitted | — | 64,651.298 |
| 7 | — | 57,103,936 | 84.82% | 7.94% | Not fitted | — | 124,182.539 |
| 7.600 | Circular-grid entry (target) | 54,339,830 | 80.71% | 7.94% | Not fitted | — | 157,608.813 |
| 8 | — | 52,571,824 | 78.09% | 7.94% | Not fitted | — | 178,989.195 |
| 9 | — | 48,399,396 | 71.89% | 7.94% | Not fitted | — | 229,446.056 |
| 10 | — | 44,558,119 | 66.19% | 7.94% | Not fitted | — | 275,898.346 |
| 11 | Spoke assembly underway (target) | 41,021,711 | 60.93% | 7.94% | Not fitted | — | 318,663.894 |
| 12 | — | 37,765,974 | 56.10% | 7.94% | Not fitted | — | 358,035.304 |
| 13 | — | 34,768,634 | 51.64% | 7.94% | Not fitted | — | 394,281.955 |
| 14 | — | 32,009,182 | 47.55% | 7.94% | Not fitted | — | 427,651.850 |
| 15 | — | 29,468,738 | 43.77% | 7.94% | Not fitted | — | 458,373.306 |
| 16 | — | 27,129,920 | 40.30% | 7.94% | Not fitted | — | 486,656.520 |
| 17 | — | 24,976,725 | 37.10% | 7.94% | Not fitted | — | 512,695.006 |
| 18 | — | 22,994,422 | 34.16% | 7.94% | Not fitted | — | 536,666.921 |
| 19 | — | 21,169,448 | 31.44% | 7.94% | Not fitted | — | 558,736.282 |
| 20 | — | 19,489,314 | 28.95% | 7.94% | Not fitted | — | 579,054.086 |
| 21 | — | 17,942,528 | 26.65% | 7.94% | Not fitted | — | 597,759.349 |
| 22 | — | 16,518,504 | 24.54% | 7.94% | Not fitted | — | 614,980.053 |
| 23 | Evolving toward square (target) | 15,207,500 | 22.59% | 7.94% | Not fitted | — | 630,834.021 |
| 24 | — | 14,000,545 | 20.80% | 7.94% | Not fitted | — | 645,429.727 |
| 25 | — | 12,889,382 | 19.15% | 7.94% | Not fitted | — | 658,867.035 |
| 26 | Fully squared grid (target) | 11,866,408 | 17.63% | 7.94% | Not fitted | — | 671,237.881 |
| 27 | Planet introduction / fast inward approach (target) | 10,924,624 | 16.23% | 7.94% | Not fitted | — | 682,626.909 |
| 28 | Orbit 1 entry | 10,057,587 | 14.94% | 7.94% | 300,000 | 0.000 | 693,112.040 |
| 29 | — | 9,259,362 | 13.75% | 7.94% | 237,753 | 0.231 | 702,765.014 |
| 30 | — | 8,524,490 | 12.66% | 7.94% | 188,558 | 0.446 | 711,651.877 |
| 30.263 | Orbit 1 far side | 8,341,412 | 12.39% | 7.94% | 177,463 | 0.500 | 713,865.857 |
| 31 | — | 7,847,942 | 11.66% | 7.94% | 149,779 | 0.645 | 719,833.432 |
| 32 | — | 7,225,090 | 10.73% | 7.94% | 119,276 | 0.830 | 727,365.656 |
| 32.785 | 100-km reference atmosphere boundary | 6,771,120 | 10.06% | 7.94% | 100,000 | 0.964 | 732,855.573 |
| 33 | Orbit 1 complete / orbit 2 begins | 6,651,671 | 9.88% | 7.94% | 95,324 | 1.000 | 734,300.086 |
| 34 | — | 6,123,762 | 9.10% | 7.94% | 76,540 | 1.157 | 740,684.164 |
| 35 | — | 5,637,751 | 8.37% | 7.94% | 61,825 | 1.303 | 746,561.572 |
| 36 | — | 5,190,314 | 7.71% | 7.94% | 50,308 | 1.437 | 751,972.522 |
| 36.503 | Orbit 2 far-side pass | 4,978,913 | 7.40% | 7.94% | 45,501 | 1.500 | 754,529.039 |
| 37 | — | 4,778,387 | 7.10% | 7.94% | 41,299 | 1.560 | 756,954.034 |
| 38 | — | 4,399,154 | 6.53% | 7.94% | 34,256 | 1.674 | 761,540.192 |
| 39 | — | 4,050,020 | 6.02% | 7.94% | 28,752 | 1.779 | 765,762.373 |
| 40 | Old airline-speed deadline — NOT met | 3,728,595 | 5.54% | 7.94% | 24,453 | 1.876 | 769,649.466 |
| 41 | — | 3,432,680 | 5.10% | 7.94% | 21,094 | 1.965 | 773,228.064 |
| 41.419 | Orbit 2 complete / orbit 3 begins | 3,315,794 | 4.93% | 7.94% | 19,916 | 2.000 | 774,641.612 |
| 42 | — | 3,160,251 | 4.69% | 7.94% | 18,472 | 2.047 | 776,522.653 |
| 43 | — | 2,909,444 | 4.32% | 7.94% | 16,425 | 2.122 | 779,555.772 |
| 44 | — | 2,678,542 | 3.98% | 7.94% | 14,827 | 2.192 | 782,348.174 |
| 45 | — | 2,465,966 | 3.66% | 7.94% | 13,579 | 2.256 | 784,918.963 |
| 46 | — | 2,270,262 | 3.37% | 7.94% | 12,605 | 2.315 | 787,285.729 |
| 47 | — | 2,090,090 | 3.10% | 7.94% | 11,845 | 2.369 | 789,464.663 |
| 48 | — | 1,924,217 | 2.86% | 7.94% | 11,252 | 2.419 | 791,470.674 |
| 49 | — | 1,771,510 | 2.63% | 7.94% | 10,789 | 2.465 | 793,317.485 |
| 49.808 | Orbit 3 far-side pass | 1,657,087 | 2.46% | 7.94% | 10,491 | 2.500 | 794,701.292 |
| 50 | — | 1,630,922 | 2.42% | 7.94% | 10,428 | 2.508 | 795,017.732 |
| 51 | — | 1,501,492 | 2.23% | 7.94% | 10,146 | 2.547 | 796,583.047 |
| 52 | — | 1,382,334 | 2.05% | 7.94% | 9,926 | 2.583 | 798,024.139 |
| 53 | — | 1,272,634 | 1.89% | 7.94% | 9,754 | 2.616 | 799,350.868 |
| 54 | — | 1,171,640 | 1.74% | 7.94% | 9,620 | 2.646 | 800,572.309 |
| 55 | — | 1,078,662 | 1.60% | 7.94% | 9,516 | 2.674 | 801,696.819 |
| 56 | — | 993,063 | 1.48% | 7.94% | 9,434 | 2.700 | 802,732.092 |
| 57 | — | 914,258 | 1.36% | 7.94% | 9,370 | 2.724 | 803,685.209 |
| 58 | Old surface deadline — NOT met | 841,707 | 1.25% | 7.94% | 9,321 | 2.746 | 804,562.691 |
| 59 | — | 774,914 | 1.15% | 7.94% | 9,282 | 2.766 | 805,370.541 |
| 60 | — | 713,422 | 1.06% | 7.94% | 9,252 | 2.785 | 806,114.286 |
| 61 | — | 656,811 | 0.98% | 7.94% | 9,228 | 2.802 | 806,799.013 |
| 62 | — | 604,693 | 0.90% | 7.94% | 9,209 | 2.817 | 807,429.406 |
| 63 | — | 556,711 | 0.83% | 7.93% | 9,195 | 2.832 | 808,009.777 |
| 64 | — | 512,538 | 0.76% | 7.93% | 9,184 | 2.845 | 808,544.097 |
| 65 | — | 471,870 | 0.70% | 7.93% | 9,175 | 2.858 | 809,036.021 |
| 66 | — | 434,430 | 0.65% | 7.93% | 9,168 | 2.869 | 809,488.912 |
| 67 | — | 399,961 | 0.59% | 7.93% | 9,163 | 2.879 | 809,905.870 |
| 68 | — | 368,228 | 0.55% | 7.93% | 9,159 | 2.889 | 810,289.747 |
| 69 | — | 339,014 | 0.50% | 7.93% | 9,156 | 2.898 | 810,643.166 |
| 70 | — | 312,118 | 0.46% | 7.93% | 9,153 | 2.906 | 810,968.547 |
| 71 | — | 287,357 | 0.43% | 7.93% | 9,151 | 2.913 | 811,268.114 |
| 72 | — | 264,561 | 0.39% | 7.93% | 9,149 | 2.920 | 811,543.916 |
| 73 | — | 243,574 | 0.36% | 7.93% | 9,148 | 2.926 | 811,797.839 |
| 74 | — | 224,253 | 0.33% | 7.93% | 9,147 | 2.932 | 812,031.619 |
| 75 | — | 206,466 | 0.31% | 7.93% | 9,147 | 2.938 | 812,246.856 |
| 76 | — | 190,090 | 0.28% | 7.93% | 9,146 | 2.943 | 812,445.021 |
| 77 | — | 175,014 | 0.26% | 7.93% | 9,146 | 2.947 | 812,627.469 |
| 78 | — | 161,134 | 0.24% | 7.93% | 9,145 | 2.951 | 812,795.447 |
| 79 | — | 148,356 | 0.22% | 7.93% | 9,145 | 2.955 | 812,950.104 |
| 80 | — | 136,592 | 0.20% | 7.93% | 9,145 | 2.959 | 813,092.496 |
| 81 | — | 125,762 | 0.19% | 7.93% | 9,145 | 2.962 | 813,223.599 |
| 82 | — | 115,791 | 0.17% | 7.93% | 9,144 | 2.965 | 813,344.306 |
| 83 | — | 106,612 | 0.16% | 7.93% | 9,144 | 2.968 | 813,455.444 |
| 84 | — | 98,161 | 0.15% | 7.93% | 9,144 | 2.970 | 813,557.772 |
| 85 | — | 90,381 | 0.13% | 7.93% | 9,144 | 2.973 | 813,651.989 |
| 86 | — | 83,218 | 0.12% | 7.92% | 9,144 | 2.975 | 813,738.739 |
| 87 | — | 76,624 | 0.11% | 7.92% | 9,144 | 2.977 | 813,818.615 |
| 88 | — | 70,553 | 0.10% | 7.92% | 9,144 | 2.979 | 813,892.161 |
| 89 | — | 64,964 | 0.10% | 7.92% | 9,144 | 2.980 | 813,959.881 |
| 90 | — | 59,819 | 0.09% | 7.92% | 9,144 | 2.982 | 814,022.237 |
| 91 | — | 55,082 | 0.08% | 7.92% | 9,144 | 2.983 | 814,079.655 |
| 92 | — | 50,721 | 0.08% | 7.92% | 9,144 | 2.985 | 814,132.526 |
| 93 | — | 46,706 | 0.07% | 7.92% | 9,144 | 2.986 | 814,181.211 |
| 94 | — | 43,009 | 0.06% | 7.91% | 9,144 | 2.987 | 814,226.043 |
| 95 | — | 39,606 | 0.06% | 7.91% | 9,144 | 2.988 | 814,267.327 |
| 96 | — | 36,473 | 0.05% | 7.91% | 9,144 | 2.989 | 814,305.346 |
| 97 | — | 33,589 | 0.05% | 7.91% | 9,144 | 2.990 | 814,340.357 |
| 98 | — | 30,934 | 0.05% | 7.91% | 9,144 | 2.991 | 814,372.601 |
| 99 | — | 28,489 | 0.04% | 7.90% | 9,144 | 2.991 | 814,402.295 |
| 100 | — | 26,239 | 0.04% | 7.90% | 9,144 | 2.992 | 814,429.644 |
| 101 | — | 24,167 | 0.04% | 7.90% | 9,144 | 2.993 | 814,454.833 |
| 102 | — | 22,259 | 0.03% | 7.89% | 9,144 | 2.993 | 814,478.033 |
| 103 | — | 20,503 | 0.03% | 7.89% | 9,144 | 2.994 | 814,499.402 |
| 104 | — | 18,887 | 0.03% | 7.89% | 9,144 | 2.994 | 814,519.086 |
| 105 | — | 17,398 | 0.03% | 7.88% | 9,144 | 2.995 | 814,537.218 |
| 106 | — | 16,028 | 0.02% | 7.88% | 9,144 | 2.995 | 814,553.922 |
| 107 | — | 14,766 | 0.02% | 7.87% | 9,144 | 2.996 | 814,569.310 |
| 108 | — | 13,605 | 0.02% | 7.87% | 9,144 | 2.996 | 814,583.488 |
| 109 | — | 12,536 | 0.02% | 7.86% | 9,144 | 2.996 | 814,596.551 |
| 110 | — | 11,551 | 0.02% | 7.85% | 9,144 | 2.997 | 814,608.588 |
| 111 | — | 10,645 | 0.02% | 7.85% | 9,144 | 2.997 | 814,619.680 |
| 112 | — | 9,811 | 0.01% | 7.84% | 9,144 | 2.997 | 814,629.902 |
| 113 | — | 9,043 | 0.01% | 7.83% | 9,144 | 2.997 | 814,639.324 |
| 114 | — | 8,336 | 0.01% | 7.82% | 9,144 | 2.998 | 814,648.008 |
| 115 | — | 7,685 | 0.01% | 7.81% | 9,144 | 2.998 | 814,656.014 |
| 116 | — | 7,085 | 0.01% | 7.80% | 9,144 | 2.998 | 814,663.394 |
| 117 | — | 6,533 | 0.009705% | 7.79% | 9,144 | 2.998 | 814,670.200 |
| 118 | — | 6,026 | 0.008950% | 7.78% | 9,144 | 2.998 | 814,676.476 |
| 119 | — | 5,558 | 0.008255% | 7.76% | 9,144 | 2.998 | 814,682.264 |
| 120 | — | 5,127 | 0.007616% | 7.75% | 9,144 | 2.999 | 814,687.604 |
| 121 | — | 4,731 | 0.007027% | 7.73% | 9,144 | 2.999 | 814,692.530 |
| 122 | — | 4,366 | 0.006485% | 7.71% | 9,144 | 2.999 | 814,697.076 |
| 123 | — | 4,030 | 0.005986% | 7.69% | 9,144 | 2.999 | 814,701.272 |
| 124 | — | 3,721 | 0.005527% | 7.67% | 9,144 | 2.999 | 814,705.145 |
| 125 | — | 3,436 | 0.005104% | 7.65% | 9,144 | 2.999 | 814,708.722 |
| 126 | — | 3,174 | 0.004714% | 7.63% | 9,144 | 2.999 | 814,712.025 |
| 127 | — | 2,933 | 0.004356% | 7.60% | 9,144 | 2.999 | 814,715.076 |
| 128 | — | 2,710 | 0.004026% | 7.58% | 9,144 | 2.999 | 814,717.896 |
| 129 | — | 2,506 | 0.003722% | 7.55% | 9,144 | 2.999 | 814,720.503 |
| 130 | — | 2,317 | 0.003442% | 7.52% | 9,144 | 2.999 | 814,722.913 |
| 131 | — | 2,144 | 0.003185% | 7.48% | 9,144 | 2.999 | 814,725.143 |
| 132 | — | 1,984 | 0.002948% | 7.44% | 9,144 | 3.000 | 814,727.206 |
| 133 | — | 1,838 | 0.002729% | 7.40% | 9,144 | 3.000 | 814,729.116 |
| 134 | — | 1,702 | 0.002528% | 7.36% | 9,144 | 3.000 | 814,730.885 |
| 135 | — | 1,578 | 0.002343% | 7.32% | 9,144 | 3.000 | 814,732.524 |
| 136 | — | 1,463 | 0.002173% | 7.27% | 9,144 | 3.000 | 814,734.043 |
| 137 | — | 1,357 | 0.002016% | 7.22% | 9,144 | 3.000 | 814,735.453 |
| 138 | — | 1,260 | 0.001872% | 7.16% | 9,144 | 3.000 | 814,736.761 |
| 139 | — | 1,171 | 0.001739% | 7.10% | 9,144 | 3.000 | 814,737.976 |
| 140 | — | 1,088 | 0.001617% | 7.04% | 9,144 | 3.000 | 814,739.105 |
| 141 | — | 1,013 | 0.001504% | 6.97% | 9,144 | 3.000 | 814,740.155 |
| 142 | — | 943 | 0.001400% | 6.89% | 9,144 | 3.000 | 814,741.132 |
| 143 | — | 879 | 0.001305% | 6.82% | 9,144 | 3.000 | 814,742.043 |
| 144 | — | 819 | 0.001217% | 6.74% | 9,144 | 3.000 | 814,742.891 |
| 145 | — | 765 | 0.001136% | 6.65% | 9,144 | 3.000 | 814,743.683 |
| 146 | — | 715 | 0.001062% | 6.56% | 9,144 | 3.000 | 814,744.422 |
| 147 | — | 669 | 0.000993% | 6.46% | 9,144 | 3.000 | 814,745.114 |
| 148 | — | 626 | 0.000930% | 6.36% | 9,144 | 3.000 | 814,745.761 |
| 148.657 | Orbit 3 complete / airline height | 600 | 0.000891% | 6.29% | 9,144 | 3.000 | 814,746.163 |

| Surface / canyon arrival | Unresolved — no fitted terrain-relative route or valid arrival time |
|---|---|
