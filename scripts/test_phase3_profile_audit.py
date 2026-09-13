"""Synthetic positive/negative witnesses for playback counter decoding."""
import base64
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location(
    "playback_profile", Path(__file__).with_name("profile-phase3-playback.py"))
profile = importlib.util.module_from_spec(spec)
spec.loader.exec_module(profile)


def frame(digits):
    rgb = bytearray(19 * 320 * 3)
    for left, digit in zip((7, 15, 23, 34), digits):
        bits = profile.DIGITS[digit]
        for row in range(5):
            for column in range(3):
                if bits & (1 << (14 - row * 3 - column)):
                    offset = ((7 + row * 2) * 320 + left + column * 2) * 3
                    rgb[offset:offset + 3] = bytes((240, 218, 156))
    # Real protocol's chunking; the decoder must not assume one RGB payload.
    return b"".join(b"\x1b_Gm=1;" + base64.b64encode(rgb[start:start + 3072]) + b"\x1b\\"
                    for start in range(0, len(rgb), 3072)) + profile.MARKER


class PlaybackCounterTests(unittest.TestCase):
    def test_digits_across_chunked_protocol(self):
        for digit in range(10):
            self.assertEqual(profile.counter(frame((0, 0, digit, digit))), digit + digit / 10)

    def test_endpoint_and_multi_digit_time(self):
        self.assertEqual(profile.counter(frame((0, 7, 7, 0))), 77)
        self.assertEqual(profile.counter(frame((1, 2, 3, 4))), 123.4)

    def test_truncated_payload_does_not_pass(self):
        with self.assertRaisesRegex(AssertionError, "Truncated"):
            profile.counter(frame((0, 3, 7, 2))[:1000])

    def test_missing_counter_does_not_pass(self):
        empty = b"\x1b_Gm=0;" + base64.b64encode(bytes(19 * 320 * 3)) + b"\x1b\\"
        with self.assertRaisesRegex(AssertionError, "Invalid timestamp"):
            profile.counter(empty)


if __name__ == "__main__":
    unittest.main()
