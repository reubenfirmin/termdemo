# Crash and performance investigation notes

## 2026-09-05: high memory use and persistent system sluggishness

Timezone: America/Costa_Rica

### Symptoms

- Codex appeared to be consuming a large amount of memory.
- The desktop remained sluggish after one unrelated Codex session was stopped.
- Sluggishness began before Google Chrome was launched; Chrome later added a
  separate CPU and memory spike and must not be treated as the original cause.

### Codex process fan-out

The largest terminal cgroup belonged to a `coalicionfloresta` processing run.
Its queue command used:

```text
python3 scripts/legal-process-queue.py ... --llm sol --llm-workers 8
```

Each document can independently use four Sol translation workers
(`_TRANSLATION_WORKERS["sol"] = 4`). The two concurrency levels therefore
multiply to a maximum of 32 simultaneous `codex exec --ephemeral` processes.

Observed at peak activity:

- 24 ephemeral Codex processes plus one interactive Codex process
- 844 Codex threads
- Approximately 150-196 MiB RSS per ephemeral invocation
- Approximately 3.8 GiB summed Codex RSS (shared pages counted repeatedly)
- Approximately 2.2 GiB unique cgroup memory
- 6.1 GiB cgroup memory peak
- 756.6 MiB cgroup swap peak
- Approximately 950 tasks in the terminal cgroup

The ephemeral processes were only 2-49 seconds old and were cycling with the
queue. There was no evidence that they were abandoned or leaked. The high
memory use was caused by intentional nested concurrency.

After the queue stopped, the cgroup fell to three processes and no ephemeral
Codex workers. Later it contained one process but retained about 1.77 GiB of
inactive file pages.

### RAM-backed `/tmp` use

`/tmp` is a tmpfs and contained about 4.25 GiB of data. The largest entry was:

```text
/tmp/coalicionfloresta-enforcement-water-build-20260904  ~3.56 GiB
```

Composition:

- `img`: about 1.93 GiB
- `assets`: about 1.18 GiB
- `docs`: about 338 MiB
- `/tmp/uv-cache`: about 402 MiB

No process had an open file under the large build directory when checked. The
directory was not deleted during the investigation. Because `/tmp` is
RAM-backed, these files consume RAM and/or swap until removed or the system is
rebooted.

### Persistent sluggishness: CPU locked at minimum frequency

The remaining sluggishness was not explained by current memory pressure:

- Memory PSI was zero.
- CPU PSI later fell close to zero.
- Swap remained around 4.8 GiB, but no sustained swap-in or swap-out was
  observed.
- Temperatures were normal: CPU approximately 50 C and NVMe approximately
  44 C.
- No recent NVMe, Btrfs, GPU-hang, or thermal error was found in the kernel
  journal.

The CPU was confirmed to be locked at its minimum frequency:

- CPU: Intel Core i5-1240P, 16 logical CPUs
- Hardware range: 400 MHz to 4.4 GHz
- All logical CPUs reported approximately 400 MHz
- A controlled CPU-bound SHA-256 workload pinned to CPU 0 did not cause any
  frequency increase
- Repeating the test after restarting `tuned.service` still produced
  approximately 400 MHz
- Governor: `powersave`
- Intel P-state: active
- Turbo: enabled (`no_turbo=0`)
- EPP: `balance_performance`
- Platform profile: `balanced`
- PowerProfiles D-Bus profile: `balanced`, with no `PerformanceDegraded`

The physical machine was running on battery, but sysfs incorrectly reported:

```text
AC: online=1
BAT0: status=Charging
```

That mismatch suggests stale or faulty firmware/embedded-controller power
state.

`tuned-adm verify` continued to fail after restarting TuneD. The detailed log
showed:

- `energy_perf_bias=7` on every CPU, while TuneD expected `normal`
- CPU governor and `energy_performance_preference` verification passed
- Boost verification read `None` while TuneD expected `1`
- `cpufreq_conservative` is built into the kernel, while TuneD attempted to
  unload it as a module

Historical package throttle counters were high (about 63,000), but neither
package nor core throttle counters increased during the controlled workload.
Temperatures were normal. This does not look like active thermal throttling;
the evidence instead points to a stuck platform/firmware power state or an
unresolved CPU policy override.

### Desktop shell observations

Dank Material Shell restarted at approximately 13:10 and briefly logged DBus
`channel full` warnings. It later used about 680 MiB, but its sampled threads
were idle and it did not explain the confirmed 400 MHz frequency lock.

### Recommended recovery

A reboot was recommended because restarting TuneD did not reset the CPU or
the incorrect AC/battery state. The reboot will also clear the RAM-backed
temporary build tree.

After reboot, verify:

1. `/sys/class/power_supply/AC/online` and `BAT0/status` match physical power.
2. CPU frequency rises above 400 MHz under a short CPU-bound workload.
3. `tuned-adm verify` and `/var/log/tuned/tuned.log` no longer show the same
   CPU policy mismatches.
4. `/tmp` usage and swap usage have fallen.
5. If the CPU remains pinned at 400 MHz, investigate Dell firmware/EC power
   limiting and collect privileged `turbostat` and
   `x86_energy_perf_policy -r` output.

