#!/usr/bin/env node
'use strict';
// Wall-time and aggregate CPU profile via existing audit instrumentation.
// No alternative executable workflow, camera, renderer or reference-image updates.
const fs=require('node:fs');
const crypto=require('node:crypto');
const {spawnSync}=require('node:child_process');
const assert=require('node:assert/strict');
const binary='target/phase0-audit/x86_64-unknown-linux-gnu/release/termdemo';
const normal='target/x86_64-unknown-linux-gnu/release/termdemo';
const args=process.argv.slice(2);
const sequence=args.includes('--sequence'),serial=args.includes('--serial');
const values=args.filter(x=>x!=='--sequence'&&x!=='--serial').map(Number);
const times=values.length?values:sequence?[0,77]:[16,26,28,36,41,57,60,65,70,75,77];
assert(times.every(t=>Number.isFinite(t)&&t>=0&&t<=77),'Times must be between 0 and 77 seconds.');
if(sequence)assert(times.length===2&&times[1]>times[0],'--sequence takes a start and end time, default 0 77.');
const queries=sequence?
  Array.from({length:Math.floor((times[1]-times[0])*30)+1},(_,i)=>[times[0]+i/30,1]):
  times.flatMap(time=>Array.from({length:3},()=>[time,1]));
const input=Buffer.alloc(16+queries.length*16);input[0]=(serial?'p':'P').charCodeAt(0);
queries.forEach(([t,a],i)=>{input.writeDoubleLE(t,16+i*16);input.writeDoubleLE(a,24+i*16);});
const run=spawnSync(binary,[],{input,timeout:300000,maxBuffer:1024*1024});
if(run.error)throw run.error;
assert.equal(run.status,0,run.stderr.toString());
assert.equal(run.stdout.subarray(0,8).toString(),'TDPROF04');
assert.equal(run.stdout.length,8+queries.length*80);
const keys=['time','aspect','wallMs','fieldCpuMs','atmosphereCpuMs','sunCpuMs','surfaceCpuMs','heightQueries','cacheMisses','renderCpuMs'];
const samples=queries.map((_,index)=>Object.fromEntries(keys.map((key,column)=>
  [key,run.stdout.readDoubleLE(8+index*80+column*8)])));
const summary=sequence?[]:times.map((time,i)=>{
  const rows=samples.slice(i*3,i*3+3);
  const med=key=>rows.map(r=>r[key]).toSorted((a,b)=>a-b)[1];
  return {time,wallMs:med('wallMs'),renderCpuMs:med('renderCpuMs'),fieldCpuMs:med('fieldCpuMs'),atmosphereCpuMs:med('atmosphereCpuMs'),
    surfaceCpuMs:med('surfaceCpuMs'),heightQueries:med('heightQueries'),cacheMisses:med('cacheMisses')};
});
const sha=path=>crypto.createHash('sha256').update(fs.readFileSync(path)).digest('hex');
const wall=samples.map(s=>s.wallMs).toSorted((a,b)=>a-b);
const distribution={frames:samples.length,medianMs:wall[Math.floor(wall.length/2)],
  p95Ms:wall[Math.floor(wall.length*.95)],p99Ms:wall[Math.floor(wall.length*.99)],
  maxMs:wall.at(-1),budgetMs:1000/30,framesOverBudget:wall.filter(ms=>ms>1000/30).length};
const readOptional=path=>{try{return fs.readFileSync(path,'utf8').trim();}catch{return null;}};
const report={normalSha256:sha(normal),auditSha256:sha(binary),
  measuredAt:new Date().toISOString(),execution:serial?'serial':'parallel',sequence,
  host:{cpuModel:readOptional('/proc/cpuinfo')?.match(/^model name\s*:\s*(.+)$/m)?.[1]??null,
    governor:readOptional('/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor'),
    affinity:readOptional('/proc/self/status')?.match(/^Cpus_allowed_list:\s*(.+)$/m)?.[1]??null,
    loadAverage:readOptional('/proc/loadavg')},
  note:'Wall time includes dispatch/assembly. Stage CPU sums ALL renderer workers. World-table startup excluded. '+
    (sequence?'Successive 1/30s camera samples, not repeated frozen frames. ':'Three repeated frames per time. ')+
    'Runs as fast as rendering allows; no terminal encoding, pacing, or claim of displayed FPS. Host settings are observed, never modified.',
  distribution,
  samples,summary};
fs.mkdirSync('target/phase3-audit',{recursive:true});
const output=`target/phase3-audit/profile${sequence?'-sequence':''}${serial?'-serial':''}.json`;
fs.writeFileSync(output,JSON.stringify(report,null,2)+'\n');
if(summary.length)console.table(summary);
console.log(distribution);
console.log(`Profile saved to ${output}.`);
