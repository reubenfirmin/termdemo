.PHONY: build check small run size audit-build audit-phase0 audit-phase1 audit-phase2 audit-phase3 audit-phase4 audit-phase5 audit-phase6 audit-phase7 audit-phase0-baseline clean

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
