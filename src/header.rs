//! Header, [HeaderExt] trait, and [TimestampExt] (timestamp) helpers.
//!
//! Every top-level VDA 5050 message (`Connection`, `InstantActions`, `Order`,
//! `State`, `Visualization`, `Factsheet`, `ZoneSet`, `Responses`) carries a
//! `Header` as its first field. The [`HeaderExt`] trait gives every one of
//! those messages a uniform `.header()`, `.header_mut()`, `.set_header(...)`
//! API so callers don't have to remember the per-message field names.
//!
//! [`TimestampExt`] extends `prost_types::Timestamp` with ISO 8601 /
//! [`SystemTime`] / `now_utc` conversions — VDA 5050 requires
//! `YYYY-MM-DDTHH:mm:ss.fffZ`. Bring `TimestampExt` into scope to call them.

use std::time::{SystemTime, UNIX_EPOCH};

use prost_types::Timestamp;

use crate::error::{HeaderError, ValidationError};
use crate::vda5050::v3::{
    Connection, Factsheet, Header, InstantActions, Order, Responses, State, Visualization, ZoneSet,
};

pub use crate::error::TimestampError;

// ---------------------------------------------------------------------------
// Header constructors + validate
// ---------------------------------------------------------------------------

impl Header {
    /// Build a new header for the given MR identity. Defaults:
    /// - `header_id = 0` (caller increments per topic),
    /// - `timestamp = None`,
    /// - `version = "3.0.0"`.
    pub fn new(
        manufacturer: impl Into<String>,
        serial_number: impl Into<String>,
    ) -> Self {
        Self {
            header_id: 0,
            timestamp: None,
            version: "3.0.0".into(),
            manufacturer: manufacturer.into(),
            serial_number: serial_number.into(),
        }
    }

    /// Chainable setter for `header_id`.
    pub fn with_header_id(mut self, id: u32) -> Self {
        self.header_id = id;
        self
    }

    /// Chainable setter for `timestamp`.
    pub fn with_timestamp(mut self, ts: Timestamp) -> Self {
        self.timestamp = Some(ts);
        self
    }

    /// Chainable setter that stamps `timestamp` with the current UTC time.
    pub fn with_timestamp_now(mut self) -> Self {
        self.timestamp = Some(Timestamp::now_utc());
        self
    }

    /// Format `timestamp` as `YYYY-MM-DDTHH:mm:ss.fffZ` (the spec's required
    /// form). Returns `None` when `timestamp` is unset.
    pub fn iso8601_timestamp(&self) -> Option<String> {
        self.timestamp.as_ref().map(Timestamp::to_iso8601_utc)
    }

    /// Validate the header per spec rules: `manufacturer`, `serial_number`,
    /// `version` are non-empty, and `version` starts with `"3."`.
    pub fn validate(&self) -> Result<(), HeaderError> {
        if self.manufacturer.is_empty() {
            return Err(HeaderError::MissingManufacturer);
        }
        if self.serial_number.is_empty() {
            return Err(HeaderError::MissingSerialNumber);
        }
        if self.version.is_empty() {
            return Err(HeaderError::MissingVersion);
        }
        if !self.version.starts_with("3.") {
            return Err(HeaderError::UnsupportedVersion(self.version.clone()));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// HeaderExt — uniform accessor for the 8 header-bearing top-level messages.
// ---------------------------------------------------------------------------

/// Uniform accessor for any top-level VDA 5050 message that carries a
/// `Header`. Implemented for every message under `vda5050::v3` whose first
/// field is `header: Option<Header>`.
pub trait HeaderExt {
    /// Borrow the inner `Header` (or `None`).
    fn header(&self) -> Option<&Header>;
    /// Mutably borrow the inner `Header` (creates an empty one if missing).
    fn header_mut(&mut self) -> &mut Header;
    /// Replace the inner `Header`.
    fn set_header(&mut self, header: Header);

    /// Convenience: validate the inner header if present; treat absence as
    /// an error so callers don't silently publish messages with no header.
    fn validate_header(&self) -> Result<(), ValidationError> {
        match self.header() {
            Some(h) => h.validate().map_err(Into::into),
            None => Err(ValidationError::Header(HeaderError::MissingManufacturer)),
        }
    }
}

macro_rules! impl_header_ext {
    ($($ty:ty),+ $(,)?) => {
        $(impl HeaderExt for $ty {
            fn header(&self) -> Option<&Header> {
                self.header.as_ref()
            }
            fn header_mut(&mut self) -> &mut Header {
                self.header.get_or_insert_with(|| Header::new("", ""))
            }
            fn set_header(&mut self, header: Header) {
                self.header = Some(header);
            }
        })+
    };
}

impl_header_ext!(
    Connection,
    InstantActions,
    Order,
    State,
    Visualization,
    Factsheet,
    ZoneSet,
    Responses,
);

// ---------------------------------------------------------------------------
// TimestampExt — ISO 8601 / SystemTime helpers.
//
// `prost_types::Timestamp` lives in an external crate; orphan rules forbid
// inherent impls, so we expose these via a trait. Bring `TimestampExt` into
// scope (`use ufms::header::TimestampExt;`) to call the methods.
// ---------------------------------------------------------------------------

pub trait TimestampExt {
    /// Current UTC timestamp, rounded to milliseconds — matches the precision
    /// of the VDA 5050 wire format.
    fn now_utc() -> Self;

    /// Format as `YYYY-MM-DDTHH:mm:ss.fffZ` (the VDA 5050 spec format).
    fn to_iso8601_utc(&self) -> String;

    /// Parse a string of the form `YYYY-MM-DDTHH:mm:ss[.fff]Z`.
    fn from_iso8601_utc(s: &str) -> Result<Self, TimestampError>
    where
        Self: Sized;

    /// Build from a [`SystemTime`].
    fn from_system_time(t: SystemTime) -> Self;

    /// Convert to a [`SystemTime`]. Saturates to `UNIX_EPOCH` on negative
    /// timestamps.
    fn to_system_time(&self) -> SystemTime;
}

impl TimestampExt for Timestamp {
    fn now_utc() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let ms = now.as_millis() as i64;
        Self {
            seconds: ms / 1000,
            nanos: ((ms % 1000) * 1_000_000) as i32,
        }
    }

    fn to_iso8601_utc(&self) -> String {
        let secs = self.seconds;
        let sub_ms = self.nanos.rem_euclid(1_000_000_000) / 1_000_000; // 0..=999 ms
        let (y, mo, d, h, mi, s) = epoch_secs_to_ymdhms(secs);
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
            y, mo, d, h, mi, s, sub_ms
        )
    }

    fn from_iso8601_utc(s: &str) -> Result<Self, TimestampError> {
        let s = s
            .strip_suffix('Z')
            .ok_or_else(|| TimestampError::BadFormat(s.to_string()))?;
        if s.len() < 19 || s.as_bytes()[10] != b'T' {
            return Err(TimestampError::BadFormat(s.to_string()));
        }
        let date = &s[..10];
        let time = &s[11..];
        let (y, mo, d) = parse_ymd(date)?;
        let (h, mi, s_sec, ms) = parse_hmsms(time)?;
        let days = days_from_civil(y, mo as u8, d as u8);
        let epoch_secs =
            days as i64 * 86_400 + (h as i64) * 3600 + (mi as i64) * 60 + s_sec as i64;
        Ok(Self {
            seconds: epoch_secs,
            nanos: (ms as i32) * 1_000_000,
        })
    }

    fn from_system_time(t: SystemTime) -> Self {
        let d = t
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|e| e.duration());
        let ms = d.as_millis() as i64;
        Self {
            seconds: ms / 1000,
            nanos: ((ms % 1000) * 1_000_000) as i32,
        }
    }

    fn to_system_time(&self) -> SystemTime {
        if self.seconds < 0 {
            return UNIX_EPOCH;
        }
        UNIX_EPOCH
            + std::time::Duration::from_secs(self.seconds as u64)
            + std::time::Duration::from_nanos(self.nanos.max(0) as u64)
    }
}

// ---------------------------------------------------------------------------
// Internal: civil-from-days / days-from-civil via Howard Hinnant's algorithm,
// ---------------------------------------------------------------------------
// std has no civil-from-days; we inline the proleptic-Gregorian conversion.

fn epoch_secs_to_ymdhms(secs: i64) -> (i32, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86_400);
    let sec_of_day = secs.rem_euclid(86_400) as u32;
    let h = sec_of_day / 3600;
    let mi = (sec_of_day % 3600) / 60;
    let s = sec_of_day % 60;
    let (y, mo, d) = civil_from_days(days);
    (y, mo, d, h, mi, s)
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    // Howard Hinnant's algorithm — inverse of days_from_civil.
    // Era length = 146097 days = 400 years of the proleptic Gregorian calendar.
    let z = z + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = (if m <= 2 { y + 1 } else { y }) as i32;
    (y, m, d)
}

fn days_from_civil(y: i32, m: u8, d: u8) -> i64 {
    // Howard Hinnant's days_from_civil.
    let y = if m as u32 <= 2 { y - 1 } else { y } as i64;
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = (y - era * 400) as u64; // [0, 399]
    let m = m as i32;
    let d = d as i32;
    let doy = ((153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1) as u64;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146_097 + (doe as i64) - 719_468
}

fn parse_ymd(s: &str) -> Result<(i32, u32, u32), TimestampError> {
    let mut parts = s.split('-');
    let y = parts
        .next()
        .ok_or_else(|| TimestampError::BadComponent(s.to_string()))?
        .parse::<i32>()
        .map_err(|_| TimestampError::BadComponent(s.to_string()))?;
    let mo = parts
        .next()
        .ok_or_else(|| TimestampError::BadComponent(s.to_string()))?
        .parse::<u32>()
        .map_err(|_| TimestampError::BadComponent(s.to_string()))?;
    let d = parts
        .next()
        .ok_or_else(|| TimestampError::BadComponent(s.to_string()))?
        .parse::<u32>()
        .map_err(|_| TimestampError::BadComponent(s.to_string()))?;
    if parts.next().is_some() {
        return Err(TimestampError::BadFormat(s.to_string()));
    }
    Ok((y, mo, d))
}

fn parse_hmsms(s: &str) -> Result<(u32, u32, u32, u32), TimestampError> {
    let (hms, ms) = match s.split_once('.') {
        Some((hms, frac)) => {
            // accept `123Z`-style or just `123` for ms fraction.
            let frac = frac.trim_end_matches('Z');
            let ms: u32 = frac
                .parse()
                .map_err(|_| TimestampError::BadComponent(s.to_string()))?;
            (hms, ms)
        }
        None => (s, 0u32),
    };
    let mut parts = hms.split(':');
    let h: u32 = parts
        .next()
        .ok_or_else(|| TimestampError::BadComponent(s.to_string()))?
        .parse()
        .map_err(|_| TimestampError::BadComponent(s.to_string()))?;
    let mi: u32 = parts
        .next()
        .ok_or_else(|| TimestampError::BadComponent(s.to_string()))?
        .parse()
        .map_err(|_| TimestampError::BadComponent(s.to_string()))?;
    let sec: u32 = parts
        .next()
        .ok_or_else(|| TimestampError::BadComponent(s.to_string()))?
        .parse()
        .map_err(|_| TimestampError::BadComponent(s.to_string()))?;
    if parts.next().is_some() {
        return Err(TimestampError::BadFormat(s.to_string()));
    }
    Ok((h, mi, sec, ms))
}
