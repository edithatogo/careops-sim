"""Reference-only clock conversions for the proposed CareOps ED profile v1.

This module is a contract fixture, not the Kairos runtime clock adapter. Fixed
configuration durations use exact decimal seconds; sampled durations preserve
the exact represented input and round upward at the model boundary.
"""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from decimal import Decimal, InvalidOperation
from fractions import Fraction
import math
import re


TICKS_PER_SECOND = 1_000_000_000
U128_MAX = (1 << 128) - 1
CONVERSION_VERSION = "careops-ed-clock-v1"


class ClockConversionError(ValueError):
    """Input cannot be represented under the proposed clock profile."""


@dataclass(frozen=True)
class SampledDuration:
    ticks: int
    sampler_version: str
    conversion_version: str
    rounded_up: bool
    rounded_draw_count: int
    rounding_error_ns: Fraction
    input_precision: str
    exact_ns_numerator: int
    exact_ns_denominator: int


@dataclass(frozen=True)
class TimestampConversion:
    ticks: int
    origin_utc: str
    original_timestamp: str
    source_zone: str | None
    source_precision_digits: int


def _decimal_value(value: str | int | Decimal) -> Decimal:
    if isinstance(value, bool) or isinstance(value, float):
        raise ClockConversionError("fixed durations require an exact decimal input")
    if isinstance(value, Decimal):
        result = value
    elif isinstance(value, (str, int)):
        try:
            result = Decimal(value)
        except (InvalidOperation, ValueError):
            raise ClockConversionError("invalid decimal duration") from None
    else:
        raise ClockConversionError("fixed durations require str, int or Decimal")
    if not result.is_finite():
        raise ClockConversionError("duration must be finite")
    if result < 0:
        raise ClockConversionError("duration must be nonnegative")
    return result


def fixed_seconds_to_ticks(seconds: str | int | Decimal) -> int:
    """Convert an exact decimal duration to nanosecond ticks without rounding."""
    value = _decimal_value(seconds)
    scaled = Fraction(value) * TICKS_PER_SECOND
    if scaled.denominator != 1:
        raise ClockConversionError("duration is not an integral number of nanoseconds")
    ticks = scaled.numerator
    if ticks > U128_MAX:
        raise ClockConversionError("duration exceeds u128 ticks")
    return ticks


def _sample_fraction(value: str | int | float | Decimal | Fraction) -> tuple[Fraction, str]:
    if isinstance(value, bool):
        raise ClockConversionError("sample must be a finite nonnegative number")
    if isinstance(value, Fraction):
        return value, "exact_rational"
    if isinstance(value, float):
        if not math.isfinite(value):
            raise ClockConversionError("sample must be finite")
        return Fraction(*value.as_integer_ratio()), "binary_float_exact_ratio"
    if isinstance(value, Decimal):
        decimal = _decimal_value(value)
        return Fraction(decimal), f"decimal_exact_{max(0, -decimal.as_tuple().exponent)}_places"
    if isinstance(value, (str, int)):
        decimal = _decimal_value(value)
        return Fraction(decimal), f"decimal_exact_{max(0, -decimal.as_tuple().exponent)}_places"
    raise ClockConversionError("unsupported sampled duration type")


def sampled_seconds_to_ticks(
    seconds: str | int | float | Decimal | Fraction,
    *,
    sampler_version: str,
    conversion_version: str = CONVERSION_VERSION,
) -> SampledDuration:
    """Ceil one sampled duration to ticks, preserving zero and its error record."""
    if not sampler_version:
        raise ClockConversionError("sampler_version is required")
    exact_seconds, precision = _sample_fraction(seconds)
    if exact_seconds < 0:
        raise ClockConversionError("sample must be nonnegative")
    exact_ns = exact_seconds * TICKS_PER_SECOND
    ticks = (exact_ns.numerator + exact_ns.denominator - 1) // exact_ns.denominator
    if ticks > U128_MAX:
        raise ClockConversionError("sampled duration exceeds u128 ticks")
    error = Fraction(ticks) - exact_ns
    return SampledDuration(
        ticks=ticks,
        sampler_version=sampler_version,
        conversion_version=conversion_version,
        rounded_up=error > 0,
        rounded_draw_count=int(error > 0),
        rounding_error_ns=error,
        input_precision=precision,
        exact_ns_numerator=exact_ns.numerator,
        exact_ns_denominator=exact_ns.denominator,
    )


_TIMESTAMP_RE = re.compile(
    r"^(?P<date>\d{4}-\d{2}-\d{2})[T ](?P<time>\d{2}:\d{2}:\d{2})"
    r"(?:\.(?P<fraction>\d+))?(?P<offset>Z|[+-]\d{2}:\d{2})?$"
)


def _parse_timestamp(value: str) -> tuple[datetime, int, int, str | None]:
    if not isinstance(value, str):
        raise ClockConversionError("timestamp must be ISO-8601 text")
    match = _TIMESTAMP_RE.fullmatch(value)
    if not match:
        raise ClockConversionError("timestamp must include an ISO date and whole seconds")
    fraction_text = match.group("fraction") or ""
    if len(fraction_text) > 9 and any(char != "0" for char in fraction_text[9:]):
        raise ClockConversionError("timestamp has sub-nanosecond precision")
    fraction_ns_int = int((fraction_text[:9] + "0" * 9)[:9] or "0")
    offset_text = match.group("offset")
    iso = f"{match.group('date')}T{match.group('time')}"
    try:
        whole = datetime.fromisoformat(iso)
    except ValueError as exc:
        raise ClockConversionError(f"invalid timestamp: {exc}") from None
    return whole, fraction_ns_int, len(fraction_text), offset_text


def _offset_tz(offset: str) -> timezone:
    if offset == "Z":
        return timezone.utc
    sign = 1 if offset[0] == "+" else -1
    hours, minutes = map(int, offset[1:].split(":"))
    if hours > 23 or minutes > 59:
        raise ClockConversionError("invalid UTC offset")
    return timezone(sign * timedelta(hours=hours, minutes=minutes))


def _utc_origin(origin_utc: str) -> tuple[datetime, int]:
    whole, fraction_ns, _, offset_text = _parse_timestamp(origin_utc)
    if offset_text is None:
        raise ClockConversionError("origin must have an explicit UTC offset")
    offset = _offset_tz(offset_text)
    if offset.utcoffset(None) != timedelta(0):
        raise ClockConversionError("origin must be UTC")
    return whole.replace(tzinfo=timezone.utc), fraction_ns


def timestamp_to_ticks(
    timestamp: str,
    origin_utc: str,
    *,
    source_zone: str | None = None,
) -> TimestampConversion:
    """Convert an absolute timestamp to checked elapsed nanosecond ticks.

    This bounded fixture accepts only timestamps carrying an explicit numeric
    offset or ``Z``. ``source_zone`` is retained as lineage metadata. Resolving
    naive IANA-zone timestamps, DST folds and gaps is intentionally unimplemented.
    """
    origin, origin_fraction_ns = _utc_origin(origin_utc)
    local, fraction_ns, precision_digits, offset_text = _parse_timestamp(timestamp)
    if offset_text is None:
        raise ClockConversionError(
            "timestamp requires an explicit UTC offset; naive zone and DST resolution are unimplemented"
        )
    zone = _offset_tz(offset_text)
    aware = local.replace(tzinfo=zone)

    utc = aware.astimezone(timezone.utc)
    delta = utc - origin
    whole_ns = (delta.days * 86_400 + delta.seconds) * TICKS_PER_SECOND + delta.microseconds * 1_000
    ticks = whole_ns + fraction_ns - origin_fraction_ns
    if ticks < 0:
        raise ClockConversionError("timestamp precedes the declared origin")
    if ticks > U128_MAX:
        raise ClockConversionError("timestamp offset exceeds u128 ticks")
    return TimestampConversion(
        ticks=ticks,
        origin_utc=origin_utc,
        original_timestamp=timestamp,
        source_zone=source_zone or offset_text,
        source_precision_digits=precision_digits,
    )
