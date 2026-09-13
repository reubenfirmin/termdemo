.PHONY: build check small run size audit-build audit-controls audit-descent audit-grid-density audit-wormhole audit-speed-function audit-flight audit-trace audit-extraction audit-harness audit-study study-flight study-opening audit-phase0 audit-phase1 audit-phase2 audit-phase3 audit-phase4 audit-phase5 audit-phase6 audit-phase7 audit-phase0-baseline clean

BIN := target/x86_64-unknown-linux-gnu/release/termdemo
SMALL := target/x86_64-unknown-linux-gnu/release/termdemo-small
AUDIT_TARGET := target/phase0-audit
AUDIT_BIN := $(AUDIT_TARGET)/x86_64-unknown-linux-gnu/release/termdemo

build:
	cargo build --release

check:
	cargo check --release

small: build
	objcopy --strip-section-headers $(BIN) $(SMALL)

run: build
	$(BIN)

size: small
	wc -c $(BIN)
	wc -c $(SMALL)
	size $(BIN)

audit-build:
	cargo build --release --features phase0-audit --target-dir $(AUDIT_TARGET)

# Test the normal executable, using the existing instrumented audit build for
# state queries. No separate preview build and no updates to reference hashes.
.PHONY: audit-phase3-live profile-phase3-render
audit-phase3-live: build audit-build
	node scripts/phase3-live-audit.cjs

profile-phase3-render: build audit-build
	node scripts/profile-phase3-render.cjs

# Candidate descent measurements only; does not rebuild the normal demo.
# Passing this gate does NOT waive the independent braking/volume gates.
audit-descent: audit-build
	printf 'v' | $(AUDIT_BIN)

audit-grid-density: audit-build
	printf 'd' | $(AUDIT_BIN)

audit-wormhole: audit-build
	printf 'w' | $(AUDIT_BIN)

# Proposed scalar function only, not a pass for the active camera trajectory.
audit-speed-function: audit-build
	rustc --edition 2024 --test src/flight_speed.rs -o target/flight-speed-tests
	target/flight-speed-tests
	printf 'b' | $(AUDIT_BIN)

audit-controls: build audit-build
	printf '_' | $(AUDIT_BIN)
	PYTHONDONTWRITEBYTECODE=1 python3 -c "from pathlib import Path; from scripts.phase0_audit import check_controls; check_controls(Path('$(BIN)').resolve())"

# Numerical/depth gates only; never run or regenerate the stored image baseline.
# Source locks and playback checks are gates, not short-circuiting prerequisites.
# Every independent gate runs; any failure still makes the aggregate fail.
audit-flight: build audit-build
	PYTHONDONTWRITEBYTECODE=1 python3 scripts/flight_audit.py suite --output target/flight-audit/gates.json

# Dense actual-camera motion, including the complete automated tail. No proposal
# is substituted for production positions and no image baseline is regenerated.
audit-trace: audit-build
	PYTHONDONTWRITEBYTECODE=1 python3 scripts/flight_audit.py capture --output target/flight-audit

audit-extraction:
	PYTHONDONTWRITEBYTECODE=1 python3 scripts/rust_sources.py

audit-harness:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts -p 'test_*audit.py'

# OFFLINE phase-2 calculation; does not install a speed law or resize the world.
# Numerical tests cannot approve the unresolved opening-star/framing/render gates.
audit-study:
	node --test scripts/test-phase2-flight-study.cjs

study-flight: audit-build audit-study
	node scripts/phase2-flight-study.cjs --markdown

# Complete fixed-star catalogue and rejected exit-route screening; not a pass
# for protected opening pixels or authority to hide/move foreground objects.
study-opening: audit-build audit-study
	node scripts/phase2-opening-study.cjs

audit-phase0: build audit-build
	python3 scripts/phase0_audit.py $(BIN) $(AUDIT_BIN) tests/phase0_frames.sha256

audit-phase1: audit-phase0
	printf '@' | $(AUDIT_BIN)

audit-phase2: audit-phase1
	printf '\043' | $(AUDIT_BIN)

audit-phase3: audit-phase2
	printf '^' | $(AUDIT_BIN)

audit-phase4: audit-phase3
	printf '\045' | $(AUDIT_BIN)

audit-phase5: audit-phase4
	printf '&' | $(AUDIT_BIN)

audit-phase6: audit-phase5
	printf '*' | $(AUDIT_BIN)

audit-phase7: audit-phase6
	printf '+' | $(AUDIT_BIN)

audit-phase0-baseline: build
	python3 scripts/phase0_audit.py $(BIN) $(BIN) tests/phase0_frames.sha256 --record

clean:
	cargo clean
