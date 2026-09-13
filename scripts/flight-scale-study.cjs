// WITHDRAWN multiplier-first study. See PLAN.md for the current decision.
// Offline history only. No production imports, scene rendering or file writes.
// node scripts/flight-scale-study.cjs reproduces the rejected scalar study.
const METRES_PER_OLD_WORLD_UNIT = 13_500 * 1_609.344 / 0.115;
const START = 4;
const FLOOR = 133;
const POWER = 10;
function integral(f, a, b, step = 0.005) {
  if (a === b) return 0;
  const n = Math.ceil((b - a) / step / 2) * 2;
  const h = (b - a) / n;
  let sum = f(a) + f(b);
  for (let i = 1; i < n; i++) sum += (i % 2 ? 4 : 2) * f(a + i * h);
  return sum * h / 3;
}
function solve(f, value, lo, hi) {
  if (f(lo) > value || f(hi) < value) throw Error('unbracketed study solve');
  for (let i = 0; i < 64; i++) {
    const mid = (lo + hi) / 2;
    if (f(mid) < value) lo = mid; else hi = mid;
  }
  return (lo + hi) / 2;
}
function candidate(reduction, planetRadius) {
  const exit = 18 * METRES_PER_OLD_WORLD_UNIT / reduction;
  const L = Math.log(exit / FLOOR);
  const a = (40 - START) / Math.pow(Math.log(exit / 600) / Math.log(600 / FLOOR), 1 / POWER);
  function speed(t) {
    const y = Math.pow(Math.max(0, t - START) / a, POWER);
    return exit * Math.exp(-L * y / (1 + y));
  }
  function rate(t) {
    const tau = Math.max(0, t - START);
    if (!tau) return 0;
    const y = Math.pow(tau / a, POWER);
    return L * POWER * y / tau / (1 + y) ** 2;
  }
  const distance = (from, to) => integral(speed, from, to);
  const peak = START + a * Math.pow((POWER - 1) / (POWER + 1), 1 / POWER);
  let worstOneSecond = { loss: 0, start: START };
  let worst100ms = { loss: 0, start: START };
  let maxAcceleration = { magnitude: 0, time: START };
  for (let i = START * 1920; i <= 58 * 1920; i++) {
    const t = i / 1920;
    if (speed(t) > speed(Math.max(START, t - 1 / 1920)) + 1e-8) throw Error('nonmonotone');
    const deceleration = speed(t) * rate(t);
    if (deceleration > maxAcceleration.magnitude) maxAcceleration = { magnitude: deceleration, time: t };
    if (t <= 57) {
      const loss = 100 * (1 - speed(t + 1) / speed(t));
      if (loss > worstOneSecond.loss) worstOneSecond = { loss, start: t };
    }
    if (t <= 57.9) {
      const loss = 100 * (1 - speed(t + .1) / speed(t));
      if (loss > worst100ms.loss) worst100ms = { loss, start: t };
    }
  }
  const tail = distance(40, 58);
  const groundVertical = solve(t => distance(40, t), 9124, 40, 180);
  const groundAt30Degrees = solve(t => distance(40, t), 18248, 40, 250);
  const orbitDistance = distance(28, 40);
  const thetaEnd = 6 * Math.PI;
  const endRadius = planetRadius + 9144;
  // Orbit-only feasibility witness, NOT the final camera route: a monotone
  // cubic-clearance spiral with three revolutions consuming the scalar arc.
  function orbitArc(r0, theta = thetaEnd) {
    return integral(phi => {
      const u = 1 - phi / thetaEnd;
      const r = endRadius + (r0 - endRadius) * u ** 3;
      const dr = -3 * (r0 - endRadius) * u ** 2 / thetaEnd;
      return Math.hypot(r, dr);
    }, 0, theta, .002);
  }
  const r0 = solve(r => orbitArc(r), orbitDistance, endRadius, 100 * endRadius);
  const orbits = [1, 2, 3].map(n => {
    const required = orbitArc(r0, n * 2 * Math.PI);
    const t = n === 3 ? 40 : solve(t => distance(28, t), required, 28, 40);
    return { n, t, altitude: 9144 + (r0 - endRadius) * (1 - n / 3) ** 3, speed: speed(t) };
  });
  const rows = [];
  for (let t = 5; t <= 58; t++) rows.push({ t, speed: speed(t),
    remainingPercent: 100 * speed(t) / speed(5),
    lossPercent: t === 5 ? null : 100 * (1 - speed(t) / speed(t - 1)),
    distanceFrom5: distance(5, t) });
  const keyframes = [5, 7.6, 11, 16, 20, 23, 26, 27, 28, 31, 34, 40, 58]
    .map(t => ({ t, speed: speed(t), routeDistanceFrom5: distance(5, t) }));
  const dimensions = {
    planetRadius, planetDiameter: 2 * planetRadius,
    circularRadius: 2.53 * METRES_PER_OLD_WORLD_UNIT / reduction,
    squareWidth: 4 * 2.53 * METRES_PER_OLD_WORLD_UNIT / reduction,
    regularPitch: 1.5 * METRES_PER_OLD_WORLD_UNIT / reduction,
    entryPitch: 3.287 * METRES_PER_OLD_WORLD_UNIT / reduction,
    minimumStarDistance: 5e11,
  };
  const integrationCheck = Math.abs(distance(5, 58) - integral(speed, 5, 58, .0025));
  return { reduction, START, POWER, FLOOR, exit, L, a, peak, peakRate: rate(peak),
    worstOneSecond, worst100ms, maxAcceleration, tail, groundVertical, groundAt30Degrees,
    orbitDistance, orbitEntryRadius: r0,
    orbitEntryRadialAngleDegrees: Math.atan(3 * (r0 - endRadius) / thetaEnd / r0) * 180 / Math.PI,
    orbits, rows, keyframes, dimensions, integrationCheck };
}
const result = {
  status: 'WITHDRAWN HISTORICAL STUDY: not the current proposal. See PLAN.md for the non-miniature planet and five-second reference lap at INITIAL speed. None of these candidates satisfies all inherited descent/timing constraints.',
  candidates: [candidate(100, 270000), candidate(1000, 81000), candidate(10000, 17000)],
  bounds: {
    maximumTailDistanceAbsoluteEaseOut: 18 * (600 + 133) / 2,
    maximumTailDistanceFractionalEaseOut: 18 * (600 - 133) / Math.log(600 / 133),
    minimumVerticalTimeFractionalEaseOut: 9124 * Math.log(600 / 133) / (600 - 133),
    minimum40SpeedFor58VerticalFractionalEaseOut: solve(v => 18 * (v - 133) / Math.log(v / 133), 9124, 600, 10000),
    minimum40SpeedFor58At30DegreesFractionalEaseOut: solve(v => 18 * (v - 133) / Math.log(v / 133), 18248, 600, 10000),
  }
};
console.log(JSON.stringify(result, null, 2));
