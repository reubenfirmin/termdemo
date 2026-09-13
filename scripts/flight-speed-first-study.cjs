// WITHDRAWN: user rejected the miniature planet and scaled viewing height.
// Historical offline calculation only: no production imports, rendering, or file writes.
// node scripts/flight-speed-first-study.cjs [--markdown]
// Physical speeds are INPUTS. No old world-unit conversion enters this study.
const assert = require('node:assert/strict');
const EXIT_TIME = 5;
const EXIT_SPEED = 30000;
const AIRCRAFT_TIME = 40;
const AIRCRAFT_SPEED = 600;
const SURFACE_TIME = 58;
const SURFACE_SPEED = 133;
const START = 4;
const ORBIT_START = 28;
// These are EXPLICIT proposed revisions, NOT the inherited 9,144-m altitude.
const PLANET_RADIUS = 650;
const AIRCRAFT_VIEW_HEIGHT = 50;
const SURFACE_CLEARANCE = 20;

function integral(f, from, to, step = 0.005) {
  if (from === to) return 0;
  const n = 2 * Math.ceil((to - from) / step / 2);
  const h = (to - from) / n;
  let sum = f(from) + f(to);
  for (let i = 1; i < n; i++) sum += (i % 2 ? 4 : 2) * f(from + i * h);
  return sum * h / 3;
}
function solve(f, target, lo, hi) {
  assert(f(lo) <= target && f(hi) >= target, 'unbracketed solve');
  for (let i = 0; i < 60; i++) {
    const mid = (lo + hi) / 2;
    if (f(mid) < target) lo = mid; else hi = mid;
  }
  return (lo + hi) / 2;
}

// v(t) = V4 exp(-L (t-4)^2 / (a2 + (t-4)^2)).
// Solve its three constants from the three declared physical speeds.
const x1 = (EXIT_TIME - START) ** 2;
const x2 = (AIRCRAFT_TIME - START) ** 2;
const x3 = (SURFACE_TIME - START) ** 2;
const logAircraft = Math.log(EXIT_SPEED / AIRCRAFT_SPEED);
const logSurface = Math.log(EXIT_SPEED / SURFACE_SPEED);
const ratio = logSurface / logAircraft * (x2 - x1) / (x3 - x1);
const a2 = (ratio * x3 - x2) / (1 - ratio);
const L = logAircraft * (a2 + x2) * (a2 + x1) / (a2 * (x2 - x1));
const V4 = EXIT_SPEED * Math.exp(L * x1 / (a2 + x1));
assert(a2 > 0 && L > 0 && EXIT_SPEED < 100000);
function speed(t) {
  const tau = t - START;
  return V4 * Math.exp(-L * tau * tau / (a2 + tau * tau));
}
function rate(t) {
  const tau = t - START;
  return 2 * L * a2 * tau / (a2 + tau * tau) ** 2;
}
function rateDerivative(t) {
  const tau = t - START;
  return 2 * L * a2 * (a2 - 3 * tau * tau) / (a2 + tau * tau) ** 3;
}
const distance = (from, to) => integral(speed, from, to);
for (const [t, value] of [[5, 30000], [40, 600], [58, 133]]) {
  assert(Math.abs(speed(t) - value) < 1e-8);
}

// An orbit-only, reference-sphere geometry witness. It is NOT a full approach,
// camera, displaced-terrain route, or new time-dependent altitude controller.
// ell is arc length after the 28-second spatial entry point.
// r(ell) = Rp + 20 + H (1 - ell / totalOrbitArc)^2.
// theta'(ell) = sqrt(1 - r'(ell)^2) / r(ell) makes |P'(ell)| = 1.
const arc40 = distance(ORBIT_START, AIRCRAFT_TIME);
const totalOrbitArc = distance(ORBIT_START, SURFACE_TIME);
const H = (AIRCRAFT_VIEW_HEIGHT - SURFACE_CLEARANCE) / (1 - arc40 / totalOrbitArc) ** 2;
const height = ell => SURFACE_CLEARANCE + H * (1 - ell / totalOrbitArc) ** 2;
const radialDerivative = ell => -2 * H / totalOrbitArc * (1 - ell / totalOrbitArc);
const angularDerivative = ell => Math.sqrt(1 - radialDerivative(ell) ** 2) / (PLANET_RADIUS + height(ell));
const phase = ell => integral(angularDerivative, 0, ell, 0.25);
assert(Math.abs(radialDerivative(0)) < 1);

const rows = [];
for (let t = 5; t <= 58; t++) {
  const ell = t >= ORBIT_START ? distance(ORBIT_START, t) : null;
  rows.push({ t, speed: speed(t), remainingPercent: 100 * speed(t) / speed(5),
    previousSecondLossPercent: t === 5 ? null : 100 * (1 - speed(t) / speed(t - 1)),
    arcFromExit: distance(5, t), referenceSphereHeight: ell === null ? null : height(ell),
    revolutions: ell === null ? null : phase(ell) / (2 * Math.PI) });
}
const orbits = [];
let previousOrbitEnd = ORBIT_START;
for (let n = 1; n <= Math.floor(phase(totalOrbitArc) / (2 * Math.PI)); n++) {
  const ell = solve(phase, n * 2 * Math.PI, 0, totalOrbitArc);
  const t = solve(t => distance(ORBIT_START, t), ell, ORBIT_START, SURFACE_TIME);
  orbits.push({ n, t, duration: t - previousOrbitEnd, height: height(ell), speed: speed(t) });
  previousOrbitEnd = t;
}

const worst = {
  oneSecond: { lossPercent: 0, t: 5 }, tenthSecond: { lossPercent: 0, t: 5 },
  scalarDeceleration: { value: 0, t: 5 }, scalarSecondDerivative: { value: 0, t: 5 },
  maxUnitTangentError: 0,
};
for (let i = 5 * 1920; i <= 58 * 1920; i++) {
  const t = i / 1920;
  assert(speed(t) <= speed(t - 1 / 1920) + 1e-9);
  for (const [key, span] of [['oneSecond', 1], ['tenthSecond', 0.1]]) {
    if (t <= 58 - span) {
      const lossPercent = 100 * (1 - speed(t + span) / speed(t));
      if (lossPercent > worst[key].lossPercent) worst[key] = { lossPercent, t };
    }
  }
  for (const [key, value] of [
    ['scalarDeceleration', speed(t) * rate(t)],
    ['scalarSecondDerivative', Math.abs(speed(t) * (rate(t) ** 2 - rateDerivative(t)))],
  ]) if (value > worst[key].value) worst[key] = { value, t };
}
for (let i = 0; i <= 20000; i++) {
  const ell = totalOrbitArc * i / 20000;
  const norm = Math.hypot(radialDerivative(ell), (PLANET_RADIUS + height(ell)) * angularDerivative(ell));
  worst.maxUnitTangentError = Math.max(worst.maxUnitTangentError, Math.abs(norm - 1));
}
assert(worst.maxUnitTangentError < 1e-12);
const integrationError = Math.abs(distance(5, 58) - integral(speed, 5, 58, 0.0025));
assert(integrationError < 1e-6);
const tail = distance(40, 58);
const keyframes = [5, 7.6, 11, 16, 20, 23, 26, 27, 28, 40, 58].map(t => ({
  t, speed: speed(t), arcFromExit: distance(5, t),
  ringPassagesPerSecondOnAxis: speed(t) / 250,
}));
const result = {
  status: 'WITHDRAWN: user rejected the miniature planet and scaled viewing height. Historical calculation only, never implemented or approved. Low post-wormhole speed, a non-miniature planet, and rapid whole-planet laps still require an explicit timing/route decision.',
  speedFunction: { START, V4, L, a2, a: Math.sqrt(a2), fractionalPeakTime: START + Math.sqrt(a2 / 3),
    peakFractionalRate: rate(START + Math.sqrt(a2 / 3)), accelerationAtSurface: -speed(58) * rate(58) },
  dimensions: { planetRadius: PLANET_RADIUS, circularDiameter: 8000, squareWidth: 12000,
    squareHeight: 12000, axialPitch: 250, regularCircularSpokes: 72, squarePerimeterLanes: 192,
    circularTransversePitch: Math.PI * 8000 / 72, squareTransversePitch: 4 * 12000 / 192,
    wellOuterRadius: 3000, nominalWellDepth: 650, minimumBackgroundStarDistance: 1e10,
    maximumPostExitTranslationalStarParallaxRadians: distance(5, 58) / 1e10 },
  orbitWitness: { referenceSphereOnly: true, H, totalOrbitArc, arc40,
    initialHeight: height(0), initialRadius: PLANET_RADIUS + height(0),
    radialEntryAngleDegrees: Math.asin(-radialDerivative(0)) * 180 / Math.PI,
    viewHeightAt40: height(arc40), surfaceClearance: height(totalOrbitArc),
    horizonDepressionAt40Degrees: Math.acos(PLANET_RADIUS / (PLANET_RADIUS + height(arc40))) * 180 / Math.PI,
    revolutionsAt40: phase(arc40) / (2 * Math.PI), orbits },
  conflicts: { inherited40Height: 9144, revised40Height: AIRCRAFT_VIEW_HEIGHT,
    requiredInheritedAltitudeDrop: 9144 - SURFACE_CLEARANCE, available40To58Arc: tail,
    minimumFirstOrbitAtInheritedHeightEvenAtExitSpeed: 2 * Math.PI * (PLANET_RADIUS + 9144) / EXIT_SPEED,
    maximum58ArcFor600To133WithFractionalEaseOut: 18 * (600 - 133) / Math.log(600 / 133) },
  keyframes, rows, worst, integrationError,
};
if (process.argv.includes('--markdown')) {
  console.log('| Time, s | Total m/s | % of 5s speed | Lost over preceding second | Distance since 5s, km | Sphere height, m | Revolutions |');
  console.log('|---:|---:|---:|---:|---:|---:|---:|');
  for (const r of rows) console.log(`| ${r.t} | ${r.speed.toFixed(1)} | ${r.remainingPercent.toFixed(2)}% | ${r.previousSecondLossPercent === null ? '—' : r.previousSecondLossPercent.toFixed(2) + '%'} | ${(r.arcFromExit / 1000).toFixed(3)} | ${r.referenceSphereHeight === null ? '—' : r.referenceSphereHeight.toFixed(1)} | ${r.revolutions === null ? '—' : r.revolutions.toFixed(3)} |`);
} else console.log(JSON.stringify(result, null, 2));
