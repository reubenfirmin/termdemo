#!/usr/bin/env node
'use strict';
// Coarse CPU profile of the production renderer via existing audit instrumentation.
// No alternative executable workflow, camera, renderer or reference-image updates.
const fs=require('node:fs');
const crypto=require('node:crypto');
const {spawnSync}=require('node:child_process');
const assert=require('node:assert/strict');
const binary='target/phase0-audit/x86_64-unknown-linux-gnu/release/termdemo';
const normal='target/x86_64-unknown-linux-gnu/release/termdemo';
const times=process.argv.length>2?process.argv.slice(2).map(Number):[16,26,28,36,41,57,60,65,70,75,77];
assert(times.every(t=>Number.isFinite(t)&&t>=0&&t<=77),'Times must be between 0 and 77 seconds.');
const queries=times.flatMap(time=>Array.from({length:3},()=>[time,1]));
const input=Buffer.alloc(16+queries.length*16);input[0]='P'.charCodeAt(0);
queries.forEach(([t,a],i)=>{input.writeDoubleLE(t,16+i*16);input.writeDoubleLE(a,24+i*16);});
const run=spawnSync(binary,[],{input,timeout:60000,maxBuffer:1024*1024});
if(run.error)throw run.error;
assert.equal(run.status,0,run.stderr.toString());
assert.equal(run.stdout.subarray(0,8).toString(),'TDPROF03');
assert.equal(run.stdout.length,8+queries.length*72);
const keys=['time','aspect','cpuMs','fieldMs','atmosphereMs','sunMs','surfaceMs','heightQueries','cacheMisses'];
const samples=queries.map((_,index)=>Object.fromEntries(keys.map((key,column)=>
  [key,run.stdout.readDoubleLE(8+index*72+column*8)])));
const summary=times.map((time,i)=>{
  const rows=samples.slice(i*3,i*3+3);
  const med=key=>rows.map(r=>r[key]).toSorted((a,b)=>a-b)[1];
  return {time,cpuMs:med('cpuMs'),fieldMs:med('fieldMs'),atmosphereMs:med('atmosphereMs'),
    surfaceMs:med('surfaceMs'),heightQueries:med('heightQueries'),cacheMisses:med('cacheMisses')};
});
const sha=path=>crypto.createHash('sha256').update(fs.readFileSync(path)).digest('hex');
const report={normalSha256:sha(normal),auditSha256:sha(binary),
  note:'CPU time; startup excluded. Three repeated frames per time, not a claim of sustained playback FPS.',
  samples,summary};
fs.mkdirSync('target/phase3-audit',{recursive:true});
fs.writeFileSync('target/phase3-audit/profile.json',JSON.stringify(report,null,2)+'\n');
console.table(summary);
console.log('Profile saved to target/phase3-audit/profile.json.');
