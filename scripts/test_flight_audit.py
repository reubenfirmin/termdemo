"""Negative witnesses for phase 1 reporting/extraction (not flight approval)."""
import contextlib
import io
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from flight_audit import independent_gates, position_derivative
from rust_sources import PipelineSource, declarations, declaration_digest


class IndependentGateTests(unittest.TestCase):
    def test_first_failure_does_not_hide_later_checks(self):
        seen = []
        def broken():
            seen.append("broken")
            raise RuntimeError("known source-lock failure")
        def later():
            seen.append("later")
            return "numerical result"
        with contextlib.redirect_stdout(io.StringIO()):
            result = independent_gates([("source", broken), ("numeric", later)])
        self.assertEqual(seen, ["broken", "later"])
        self.assertEqual([row["status"] for row in result], ["FAIL", "PASS"])
        self.assertIn("known source-lock failure", result[0]["output"])

    def test_timeout_is_failure_and_later_gate_runs(self):
        def timeout():
            raise subprocess.TimeoutExpired("fixture", 1)
        with contextlib.redirect_stdout(io.StringIO()):
            result = independent_gates([("timeout", timeout), ("last", lambda: "ran")])
        self.assertEqual([row["status"] for row in result], ["FAIL", "PASS"])


class PositionDerivativeTests(unittest.TestCase):
    def test_nonuniform_quadratic_positions(self):
        def row(t):
            return (t, t * t, 3 * t * t + t, -2 * t)
        result = position_derivative(row(0.7), row(1.0), row(1.6))
        for actual, expected in zip(result, (2.0, 7.0, -2.0)):
            self.assertAlmostEqual(actual, expected)

    def test_velocity_direction_cannot_be_replaced_by_speed_magnitude(self):
        actual = position_derivative((0, 0, 0, 0), (1, -2, 0, 0), (2, -4, 0, 0))
        self.assertEqual(actual, (-2, 0, 0))
        self.assertNotEqual(actual, (2, 0, 0))


class ExtractionTests(unittest.TestCase):
    def test_module_reader_keeps_original_raw_source_hash_measurements(self):
        # A reader UNIT fixture, not a claim that phase 3 is an unchanged phase-1
        # extraction. The old whole-repository source locks remain archived;
        # actual opening preservation is checked by the production binary audit.
        original = 'fn alpha() { 1 + 2; }\n\nfn beta() { 3 + 4; }\n'
        expected = hashlib.sha256(original.encode()).hexdigest()
        with tempfile.TemporaryDirectory() as directory:
            first, second = Path(directory) / 'a.rs', Path(directory) / 'b.rs'
            first.write_text('pub(super) fn alpha() { 1 + 2; }\n')
            second.write_text('pub(super) fn beta() { 3 + 4; }\n')
            pipeline = PipelineSource([second, first])
            self.assertEqual(hashlib.sha256(pipeline.block('alpha', 'beta').encode()).hexdigest(), expected)
            second.write_text('pub(super) fn beta() { 3 - 4; }\n')
            changed = PipelineSource([second, first])
            self.assertNotEqual(hashlib.sha256(changed.block('alpha', 'beta').encode()).hexdigest(), expected)

    def test_array_type_semicolon_is_not_end_of_static(self):
        source = 'static mut CACHE: [f64; 10] = [0.0; 10];\nfn next() {}\n'
        items = declarations(source)
        self.assertEqual(len(items), 2)
        self.assertIn('= [0.0; 10];', items[0]["text"])
        self.assertEqual(items[0]["end"], items[1]["start"])

    def test_braces_in_literals_and_comments_are_not_code(self):
        source = 'fn first() { let x = b"}"; /* } */ let c = \'{\'; }\nfn second() {}\n'
        self.assertEqual([item["name"] for item in declarations(source)], ["first", "second"])

    def test_module_visibility_only_is_ignored(self):
        original = 'struct Point {\n    x: f64,\n}\n'
        moved = 'pub(super) struct Point {\n    pub(super) x: f64,\n}\n'
        self.assertEqual(declaration_digest(original), declaration_digest(moved))
        self.assertNotEqual(declaration_digest(original), declaration_digest(moved.replace("f64", "f32")))

    def test_changed_constant_or_equation_fails(self):
        self.assertNotEqual(declaration_digest('const R: f64 = 0.115;'),
                            declaration_digest('const R: f64 = 0.116;'))
        self.assertNotEqual(declaration_digest('fn v(t: f64) -> f64 { t * 2.0 }'),
                            declaration_digest('fn v(t: f64) -> f64 { t * 3.0 }'))

    def test_cfg_attribute_is_part_of_guard(self):
        self.assertNotEqual(declaration_digest('fn legacy() {}'),
                            declaration_digest('#[cfg(feature = "phase0-audit")]\nfn legacy() {}'))


if __name__ == "__main__":
    unittest.main()
