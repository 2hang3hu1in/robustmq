// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use chrono::{DateTime, Local, Utc};
use core::fmt;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    ops::Add,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub fn get_current_millisecond_timestamp() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RobustMQTimestamp(SystemTime);

pub const UTC_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

impl RobustMQTimestamp {
    pub fn now() -> Self {
        RobustMQTimestamp::default()
    }

    pub fn zero() -> Self {
        RobustMQTimestamp(UNIX_EPOCH)
    }

    pub fn to_secs(&self) -> u64 {
        self.0.duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    pub fn as_micros(&self) -> u64 {
        self.0.duration_since(UNIX_EPOCH).unwrap().as_micros() as u64
    }

    pub fn as_millis(&self) -> u64 {
        self.0.duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
    }

    pub fn to_utc_string(&self, format: &str) -> String {
        DateTime::<Utc>::from(self.0).format(format).to_string()
    }

    pub fn to_local_string(&self, format: &str) -> String {
        DateTime::<Local>::from(self.0).format(format).to_string()
    }
}

impl From<u64> for RobustMQTimestamp {
    fn from(timestamp: u64) -> Self {
        RobustMQTimestamp(UNIX_EPOCH + Duration::from_micros(timestamp))
    }
}

impl From<RobustMQTimestamp> for u64 {
    fn from(timestamp: RobustMQTimestamp) -> u64 {
        timestamp.as_micros()
    }
}

impl Add<SystemTime> for RobustMQTimestamp {
    type Output = RobustMQTimestamp;

    fn add(self, other: SystemTime) -> RobustMQTimestamp {
        RobustMQTimestamp(self.0 + other.duration_since(UNIX_EPOCH).unwrap())
    }
}

impl Default for RobustMQTimestamp {
    fn default() -> Self {
        Self(SystemTime::now())
    }
}

impl fmt::Display for RobustMQTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_utc_string(UTC_TIME_FORMAT))
    }
}

impl Serialize for RobustMQTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let timestamp = self.as_micros();
        serializer.serialize_u64(timestamp)
    }
}

impl<'de> Deserialize<'de> for RobustMQTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(RobustMQTimestampVisitor)
    }
}
struct RobustMQTimestampVisitor;

impl Visitor<'_> for RobustMQTimestampVisitor {
    type Value = RobustMQTimestamp;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a microsecond timestamp as a u64")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RobustMQTimestamp::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_get() {
        let timestamp = RobustMQTimestamp::now();
        assert!(timestamp.as_micros() > 0);
    }

    #[test]
    fn test_timestamp_to_micros() {
        let timestamp = RobustMQTimestamp::from(1738405752756068);
        assert_eq!(timestamp.as_micros(), 1738405752756068);
    }

    #[test]
    fn test_timestamp_to_string() {
        let timestamp = RobustMQTimestamp::from(1738405752756068);
        assert_eq!(
            timestamp.to_utc_string("%Y-%m-%d %H:%M:%S"),
            "2025-02-01 18:29:12"
        );
    }

    #[test]
    fn test_timestamp_from_u64() {
        let timestamp = RobustMQTimestamp::from(1738405752756068);
        assert_eq!(timestamp.as_micros(), 1738405752756068);
    }
}
