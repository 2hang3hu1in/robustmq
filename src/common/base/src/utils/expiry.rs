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

use crate::utils::duration::RobustMQDuration;
use humantime::format_duration;
use humantime::Duration as HumanDuration;
use serde::de::Visitor;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::fmt::Display;
use std::iter::Sum;
use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;

/// Helper enum for various time-based expiry related functionalities
#[derive(Debug, Copy, Default, Clone, Eq, PartialEq)]
pub enum RobustMQExpiry {
    #[default]
    /// Use the default expiry time from the server
    ServerDefault,
    /// Set expiry time to given value
    ExpireDuration(RobustMQDuration),
    /// Never expire
    NeverExpire,
}

impl RobustMQExpiry {
    pub fn new(values: Option<Vec<RobustMQExpiry>>) -> Option<Self> {
        values.map(|items| items.iter().cloned().sum())
    }
}

impl From<&RobustMQExpiry> for Option<u64> {
    fn from(value: &RobustMQExpiry) -> Self {
        match value {
            RobustMQExpiry::ExpireDuration(value) => Some(value.as_micros()),
            RobustMQExpiry::NeverExpire => Some(u64::MAX),
            RobustMQExpiry::ServerDefault => None,
        }
    }
}

impl Display for RobustMQExpiry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NeverExpire => write!(f, "none"),
            Self::ServerDefault => write!(f, "server_default"),
            Self::ExpireDuration(value) => write!(f, "{value}"),
        }
    }
}

impl Sum for RobustMQExpiry {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.into_iter()
            .fold(RobustMQExpiry::NeverExpire, |acc, x| acc + x)
    }
}

impl Add for RobustMQExpiry {
    type Output = RobustMQExpiry;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (RobustMQExpiry::NeverExpire, RobustMQExpiry::NeverExpire) => {
                RobustMQExpiry::NeverExpire
            }
            (RobustMQExpiry::NeverExpire, expiry) => expiry,
            (expiry, RobustMQExpiry::NeverExpire) => expiry,
            (
                RobustMQExpiry::ExpireDuration(lhs_duration),
                RobustMQExpiry::ExpireDuration(rhs_duration),
            ) => RobustMQExpiry::ExpireDuration(lhs_duration + rhs_duration),
            (RobustMQExpiry::ServerDefault, RobustMQExpiry::ExpireDuration(_)) => {
                RobustMQExpiry::ServerDefault
            }
            (RobustMQExpiry::ServerDefault, RobustMQExpiry::ServerDefault) => {
                RobustMQExpiry::ServerDefault
            }
            (RobustMQExpiry::ExpireDuration(_), RobustMQExpiry::ServerDefault) => {
                RobustMQExpiry::ServerDefault
            }
        }
    }
}

impl FromStr for RobustMQExpiry {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = match s {
            "unlimited" | "none" | "None" | "Unlimited" => RobustMQExpiry::NeverExpire,
            "default" | "server_default" | "Default" | "Server_default" => {
                RobustMQExpiry::ServerDefault
            }
            value => {
                let duration = value.parse::<HumanDuration>().map_err(|e| format!("{e}"))?;
                if duration.as_secs() > u32::MAX as u64 {
                    return Err(format!(
                        "Value too big for expiry time, maximum value is {}",
                        format_duration(Duration::from_secs(u32::MAX as u64))
                    ));
                }

                RobustMQExpiry::ExpireDuration(RobustMQDuration::from(duration))
            }
        };

        Ok(result)
    }
}

impl From<RobustMQExpiry> for Option<u64> {
    fn from(val: RobustMQExpiry) -> Self {
        match val {
            RobustMQExpiry::ExpireDuration(value) => Some(value.as_micros()),
            RobustMQExpiry::ServerDefault => None,
            RobustMQExpiry::NeverExpire => Some(u64::MAX),
        }
    }
}

impl From<RobustMQExpiry> for u64 {
    fn from(val: RobustMQExpiry) -> Self {
        match val {
            RobustMQExpiry::ExpireDuration(value) => value.as_micros(),
            RobustMQExpiry::ServerDefault => 0,
            RobustMQExpiry::NeverExpire => u64::MAX,
        }
    }
}

impl From<Vec<RobustMQExpiry>> for RobustMQExpiry {
    fn from(values: Vec<RobustMQExpiry>) -> Self {
        let mut result = RobustMQExpiry::NeverExpire;
        for value in values {
            result = result + value;
        }
        result
    }
}

impl From<u64> for RobustMQExpiry {
    fn from(value: u64) -> Self {
        match value {
            u64::MAX => RobustMQExpiry::NeverExpire,
            0 => RobustMQExpiry::ServerDefault,
            value => RobustMQExpiry::ExpireDuration(RobustMQDuration::from(value)),
        }
    }
}

impl From<Option<u64>> for RobustMQExpiry {
    fn from(value: Option<u64>) -> Self {
        match value {
            Some(value) => match value {
                u64::MAX => RobustMQExpiry::NeverExpire,
                0 => RobustMQExpiry::ServerDefault,
                value => RobustMQExpiry::ExpireDuration(RobustMQDuration::from(value)),
            },
            None => RobustMQExpiry::NeverExpire,
        }
    }
}

impl Serialize for RobustMQExpiry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let expiry = match self {
            RobustMQExpiry::ExpireDuration(value) => value.as_micros(),
            RobustMQExpiry::ServerDefault => 0,
            RobustMQExpiry::NeverExpire => u64::MAX,
        };
        serializer.serialize_u64(expiry)
    }
}

impl<'de> Deserialize<'de> for RobustMQExpiry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(RobustMQExpiryVisitor)
    }
}

struct RobustMQExpiryVisitor;

impl Visitor<'_> for RobustMQExpiryVisitor {
    type Value = RobustMQExpiry;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a microsecond expiry as a u64")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(RobustMQExpiry::from(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::duration::SEC_IN_MICRO;

    #[test]
    fn should_parse_expiry() {
        assert_eq!(
            RobustMQExpiry::from_str("none").unwrap(),
            RobustMQExpiry::NeverExpire
        );
        assert_eq!(
            RobustMQExpiry::from_str("15days").unwrap(),
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(
                SEC_IN_MICRO * 60 * 60 * 24 * 15
            ))
        );
        assert_eq!(
            RobustMQExpiry::from_str("2min").unwrap(),
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(SEC_IN_MICRO * 60 * 2))
        );
        assert_eq!(
            RobustMQExpiry::from_str("1ms").unwrap(),
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(1000))
        );
        assert_eq!(
            RobustMQExpiry::from_str("1s").unwrap(),
            RobustMQExpiry::ExpireDuration(RobustMQDuration::ONE_SECOND)
        );
        assert_eq!(
            RobustMQExpiry::from_str("15days 2min 2s").unwrap(),
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(
                SEC_IN_MICRO * (60 * 60 * 24 * 15 + 60 * 2 + 2)
            ))
        );
    }

    #[test]
    fn should_fail_parsing_expiry() {
        let x = RobustMQExpiry::from_str("15se");
        assert!(x.is_err());
        assert_eq!(
            x.unwrap_err(),
            "unknown time unit \"se\", supported units: ns, us, ms, sec, min, hours, days, weeks, months, years (and few variations)"
        );
    }

    #[test]
    fn should_sum_expiry() {
        assert_eq!(
            RobustMQExpiry::NeverExpire + RobustMQExpiry::NeverExpire,
            RobustMQExpiry::NeverExpire
        );
        assert_eq!(
            RobustMQExpiry::NeverExpire + RobustMQExpiry::ExpireDuration(RobustMQDuration::from(3)),
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(3))
        );
        assert_eq!(
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(5)) + RobustMQExpiry::NeverExpire,
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(5))
        );
        assert_eq!(
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(5))
                + RobustMQExpiry::ExpireDuration(RobustMQDuration::from(3)),
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(8))
        );
    }

    #[test]
    fn should_sum_expiry_from_vec() {
        assert_eq!(
            vec![RobustMQExpiry::NeverExpire]
                .into_iter()
                .sum::<RobustMQExpiry>(),
            RobustMQExpiry::NeverExpire
        );
        let x = vec![
            RobustMQExpiry::NeverExpire,
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(333)),
            RobustMQExpiry::NeverExpire,
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(123)),
        ];
        assert_eq!(
            x.into_iter().sum::<RobustMQExpiry>(),
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(456))
        );
    }

    #[test]
    fn should_check_display_expiry() {
        assert_eq!(RobustMQExpiry::NeverExpire.to_string(), "none");
        assert_eq!(
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(333333000000)).to_string(),
            "3days 20h 35m 33s"
        );
    }

    #[test]
    fn should_calculate_none_from_server_default() {
        let expiry = RobustMQExpiry::ServerDefault;
        let result: Option<u64> = From::from(&expiry);
        assert_eq!(result, None);
    }

    #[test]
    fn should_calculate_u64_max_from_never_expiry() {
        let expiry = RobustMQExpiry::NeverExpire;
        let result: Option<u64> = From::from(&expiry);
        assert_eq!(result, Some(u64::MAX));
    }

    #[test]
    fn should_calculate_some_seconds_from_message_expire() {
        let duration = RobustMQDuration::new(Duration::new(42, 0));
        let expiry = RobustMQExpiry::ExpireDuration(duration);
        let result: Option<u64> = From::from(&expiry);
        assert_eq!(result, Some(42000000));
    }

    #[test]
    fn should_create_new_expiry_from_vec() {
        let some_values = vec![
            RobustMQExpiry::NeverExpire,
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(3)),
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(2)),
            RobustMQExpiry::ExpireDuration(RobustMQDuration::from(1)),
        ];
        assert_eq!(
            RobustMQExpiry::new(Some(some_values)),
            Some(RobustMQExpiry::ExpireDuration(RobustMQDuration::from(6)))
        );
        assert_eq!(RobustMQExpiry::new(None), None);
        let none_values = vec![RobustMQExpiry::ServerDefault; 10];

        assert_eq!(
            RobustMQExpiry::new(Some(none_values)),
            Some(RobustMQExpiry::ServerDefault)
        );
    }
}
