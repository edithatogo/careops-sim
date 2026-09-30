"""Contract tests for the proposed CareOps ED profile v1 clock fixture."""

import math
import unittest
from decimal import Decimal
from fractions import Fraction

from tools.ed_clock_reference import (
    ClockConversionError,
    U128_MAX,
    fixed_seconds_to_ticks,
    sampled_seconds_to_ticks,
    timestamp_to_ticks,
)


class FixedDurationTests(unittest.TestCase):
    def test_exact_decimal_scaling_and_trailing_zeroes(self):
        self.assertEqual(fixed_seconds_to_ticks("1.25"), 1_250_000_000)
        self.assertEqual(fixed_seconds_to_ticks("0.0000000010000"), 1)
        self.assertEqual(fixed_seconds_to_ticks(Decimal("0")), 0)
        maximum_seconds = f"{U128_MAX // 1_000_000_000}.{U128_MAX % 1_000_000_000:09d}"
        self.assertEqual(fixed_seconds_to_ticks(maximum_seconds), U128_MAX)

    def test_rejects_non_integral_negative_nonfinite_and_overflow(self):
        for value in ("0.0000000001", "-1", "NaN", "Infinity", str(U128_MAX + 1)):
            with self.subTest(value=value), self.assertRaises(ClockConversionError):
                fixed_seconds_to_ticks(value)
        with self.assertRaises(ClockConversionError):
            fixed_seconds_to_ticks(0.5)


class SampledDurationTests(unittest.TestCase):
    def test_zero_and_exact_sample_keep_zero_error(self):
        zero = sampled_seconds_to_ticks("0", sampler_version="fixed-v1")
        exact = sampled_seconds_to_ticks("0.000000002", sampler_version="fixed-v1")
        self.assertEqual((zero.ticks, zero.rounded_draw_count, zero.rounding_error_ns), (0, 0, 0))
        self.assertEqual((exact.ticks, exact.rounded_up, exact.rounding_error_ns), (2, False, 0))

    def test_rounds_up_from_exact_decimal_rational_and_binary_float(self):
        decimal_draw = sampled_seconds_to_ticks("0.0000000001", sampler_version="decimal-v1")
        rational_draw = sampled_seconds_to_ticks(Fraction(1, 3_000_000_000), sampler_version="rational-v1")
        float_draw = sampled_seconds_to_ticks(0.1, sampler_version="float-v1")
        self.assertEqual((decimal_draw.ticks, decimal_draw.rounded_draw_count), (1, 1))
        self.assertEqual(decimal_draw.rounding_error_ns, Fraction(9, 10))
        self.assertEqual((rational_draw.ticks, rational_draw.rounding_error_ns), (1, Fraction(2, 3)))
        self.assertTrue(float_draw.rounded_up)
        self.assertLess(float_draw.rounding_error_ns, 1)
        self.assertEqual(float_draw.input_precision, "binary_float_exact_ratio")

    def test_rejects_invalid_samples_and_records_versions(self):
        result = sampled_seconds_to_ticks(Decimal("0.25"), sampler_version="sampler-7")
        self.assertEqual(result.sampler_version, "sampler-7")
        self.assertEqual(result.conversion_version, "careops-ed-clock-v1")
        self.assertEqual(result.input_precision, "decimal_exact_2_places")
        for value in (-0.1, float("nan"), float("inf"), "-0.1", Fraction(-1, 2)):
            with self.subTest(value=value), self.assertRaises(ClockConversionError):
                sampled_seconds_to_ticks(value, sampler_version="sampler-7")
        with self.assertRaises(ClockConversionError):
            sampled_seconds_to_ticks(math.inf, sampler_version="sampler-7")
        with self.assertRaises(ClockConversionError):
            sampled_seconds_to_ticks(str(U128_MAX + 1), sampler_version="sampler-7")
        with self.assertRaises(ClockConversionError):
            sampled_seconds_to_ticks(1, sampler_version="")


class TimestampOriginTests(unittest.TestCase):
    ORIGIN = "2026-01-01T00:00:00Z"

    def test_explicit_utc_offset_and_fraction_convert_to_elapsed_ticks(self):
        value = timestamp_to_ticks(
            "2026-01-01T10:00:00.000000001+10:00",
            self.ORIGIN,
            source_zone="Australia/Brisbane",
        )
        self.assertEqual(value.ticks, 1)
        self.assertEqual(value.source_zone, "Australia/Brisbane")
        self.assertEqual(value.source_precision_digits, 9)
        self.assertEqual(value.original_timestamp, "2026-01-01T10:00:00.000000001+10:00")

    def test_offsets_and_trailing_fractional_zeroes_are_exact(self):
        value = timestamp_to_ticks("2026-01-01T01:00:00.1200000000+01:00", self.ORIGIN)
        self.assertEqual(value.ticks, 120_000_000)
        self.assertEqual(value.source_precision_digits, 10)

    def test_rejects_pre_origin_bad_origin_naive_and_subnanosecond(self):
        with self.assertRaises(ClockConversionError):
            timestamp_to_ticks("2025-12-31T23:59:59Z", self.ORIGIN)
        with self.assertRaises(ClockConversionError):
            timestamp_to_ticks("2026-01-01T00:00:00", self.ORIGIN, source_zone="Australia/Brisbane")
        with self.assertRaises(ClockConversionError):
            timestamp_to_ticks("2026-01-01T00:00:00.0000000001Z", self.ORIGIN)
        with self.assertRaises(ClockConversionError):
            timestamp_to_ticks("2026-01-01T00:00:00Z", "2026-01-01T10:00:00+10:00")
        with self.assertRaises(ClockConversionError):
            timestamp_to_ticks("2026-01-01T00:00:00Z", "2026-01-01T00:00:00")


if __name__ == "__main__":
    unittest.main()
