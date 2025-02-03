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

use humantime::format_duration;
use humantime::Duration as HumanDuration;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{
    fmt::{Display, Formatter},
    ops::Add,
    str::FromStr,
    time::Duration,
};

pub const SEC_IN_MICRO: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct RobustMQDuration {
    duration: Duration,
}

impl RobustMQDuration {
    pub const ONE_SECOND: RobustMQDuration = RobustMQDuration {
        duration: Duration::from_secs(1),
    };
}

impl RobustMQDuration {
    pub fn new(duration: Duration) -> RobustMQDuration {
        RobustMQDuration { duration }
    }

    pub fn new_from_secs(secs: u64) -> RobustMQDuration {
        RobustMQDuration {
            duration: Duration::from_secs(secs),
        }
    }

    pub fn as_human_time_string(&self) -> String {
        format!("{}", format_duration(self.duration))
    }

    pub fn as_secs(&self) -> u32 {
        self.duration.as_secs() as u32
    }

    pub fn as_secs_f64(&self) -> f64 {
        self.duration.as_secs_f64()
    }

    pub fn as_micros(&self) -> u64 {
        self.duration.as_micros() as u64
    }

    pub fn get_duration(&self) -> Duration {
        self.duration
    }

    pub fn is_zero(&self) -> bool {
        self.duration.as_secs() == 0
    }

    pub fn abs_diff(&self, other: RobustMQDuration) -> RobustMQDuration {
        RobustMQDuration {
            duration: self.duration.abs_diff(other.duration),
        }
    }
}

impl FromStr for RobustMQDuration {
    type Err = humantime::DurationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = &s.to_lowercase();
        if s == "0" || s == "unlimited" || s == "disabled" || s == "none" {
            Ok(RobustMQDuration {
                duration: Duration::new(0, 0),
            })
        } else {
            Ok(RobustMQDuration {
                duration: humantime::parse_duration(s)?,
            })
        }
    }
}

impl From<Option<u64>> for RobustMQDuration {
    fn from(byte_size: Option<u64>) -> Self {
        match byte_size {
            Some(value) => RobustMQDuration {
                duration: Duration::from_micros(value),
            },
            None => RobustMQDuration {
                duration: Duration::new(0, 0),
            },
        }
    }
}

impl From<u64> for RobustMQDuration {
    fn from(value: u64) -> Self {
        RobustMQDuration {
            duration: Duration::from_micros(value),
        }
    }
}

impl From<Duration> for RobustMQDuration {
    fn from(duration: Duration) -> Self {
        RobustMQDuration { duration }
    }
}

impl From<HumanDuration> for RobustMQDuration {
    fn from(human_duration: HumanDuration) -> Self {
        Self {
            duration: human_duration.into(),
        }
    }
}

impl From<RobustMQDuration> for u64 {
    fn from(robustmq_duration: RobustMQDuration) -> u64 {
        robustmq_duration.duration.as_micros() as u64
    }
}

impl Default for RobustMQDuration {
    fn default() -> Self {
        RobustMQDuration {
            duration: Duration::new(0, 0),
        }
    }
}

impl Display for RobustMQDuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_human_time_string())
    }
}

impl Add for RobustMQDuration {
    type Output = RobustMQDuration;

    fn add(self, rhs: Self) -> Self::Output {
        RobustMQDuration {
            duration: self.duration + rhs.duration,
        }
    }
}

impl Serialize for RobustMQDuration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.as_micros())
    }
}

struct RobustMQDurationVisitor;

impl<'de> Deserialize<'de> for RobustMQDuration {
    fn deserialize<D>(deserializer: D) -> Result<RobustMQDuration, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(RobustMQDurationVisitor)
    }
}

impl Visitor<'_> for RobustMQDurationVisitor {
    type Value = RobustMQDuration;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a duration in seconds")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(RobustMQDuration::new(Duration::from_micros(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_new() {
        let duration = Duration::new(60, 0); // 60 seconds
        let robustmq_duration = RobustMQDuration::new(duration);
        assert_eq!(robustmq_duration.as_secs(), 60);
    }

    #[test]
    fn test_as_human_time_string() {
        let duration = Duration::new(3661, 0); // 1 hour, 1 minute and 1 second
        let robustmq_duration = RobustMQDuration::new(duration);
        assert_eq!(robustmq_duration.as_human_time_string(), "1h 1m 1s");
    }

    #[test]
    fn test_long_duration_as_human_time_string() {
        let duration = Duration::new(36611233, 0); // 1year 1month 28days 1hour 13minutes 37seconds
        let robustmq_duration = RobustMQDuration::new(duration);
        assert_eq!(
            robustmq_duration.as_human_time_string(),
            "1year 1month 28days 1h 13m 37s"
        );
    }

    #[test]
    fn test_from_str() {
        let robustmq_duration: RobustMQDuration = "1h 1m 1s".parse().unwrap();
        assert_eq!(robustmq_duration.as_secs(), 3661);
    }

    #[test]
    fn test_display() {
        let duration = Duration::new(3661, 0);
        let robustmq_duration = RobustMQDuration::new(duration);
        let duration_string = format!("{}", robustmq_duration);
        assert_eq!(duration_string, "1h 1m 1s");
    }

    #[test]
    fn test_invalid_duration() {
        let result: Result<RobustMQDuration, _> = "1 hour and 30 minutes".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_seconds_duration() {
        let robustmq_duration: RobustMQDuration = "0s".parse().unwrap();
        assert_eq!(robustmq_duration.as_secs(), 0);
    }

    #[test]
    fn test_zero_duration() {
        let robustmq_duration: RobustMQDuration = "0".parse().unwrap();
        assert_eq!(robustmq_duration.as_secs(), 0);
    }

    #[test]
    fn test_unlimited() {
        let robustmq_duration: RobustMQDuration = "unlimited".parse().unwrap();
        assert_eq!(robustmq_duration.as_secs(), 0);
    }

    #[test]
    fn test_disabled() {
        let robustmq_duration: RobustMQDuration = "disabled".parse().unwrap();
        assert_eq!(robustmq_duration.as_secs(), 0);
    }

    #[test]
    fn test_add_duration() {
        let robustmq_duration1: RobustMQDuration = "6s".parse().unwrap();
        let robustmq_duration2: RobustMQDuration = "1m".parse().unwrap();
        let result: RobustMQDuration = robustmq_duration1 + robustmq_duration2;
        assert_eq!(result.as_secs(), 66);
    }
}
