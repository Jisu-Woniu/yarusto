use std::{fmt, ops::Deref, str::FromStr, time::Duration};

use serde::{
    de::{Error, Unexpected, Visitor},
    Deserialize, Serialize,
};

use crate::model::types::InvalidUnit;

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CustomDuration(pub(crate) Duration);

impl Default for CustomDuration {
    fn default() -> Self {
        Self(Duration::from_secs(1))
    }
}

impl From<Duration> for CustomDuration {
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

impl From<CustomDuration> for Duration {
    fn from(value: CustomDuration) -> Self {
        value.0
    }
}

impl Deref for CustomDuration {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for CustomDuration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(DurationVisitor)
    }
}

pub struct DurationVisitor;

impl Visitor<'_> for DurationVisitor {
    type Value = CustomDuration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "a value representing duration, ranges from 0.1s to 10s, i.e. 100ms to 10,000ms",
        )
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Duration::from_millis(match v {
            1..=10 => Ok(v * 1000),
            100..=10000 => Ok(v),
            _ => Err(Error::invalid_value(Unexpected::Unsigned(v), &self))?,
        }?)
        .into())
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Duration::from_secs_f64(match v {
            0.1..10.0 => Ok(v),
            100.0..10000.0 => Ok(v / 1000.0),
            _ => Err(Error::invalid_value(Unexpected::Float(v), &self))?,
        }?)
        .into())
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let (value, unit) = value.split_at(
            value
                .find(|c: char| !(c.is_ascii_digit() || c == '.'))
                .unwrap_or(value.len()),
        );
        let value: f64 = value.parse().map_err(|e| Error::custom(e))?;

        let unit = DurationUnit::from_str(unit).map_err(|e| Error::custom(e))?;

        Ok(Duration::try_from_secs_f64(match unit {
            DurationUnit::Unspecified => return self.visit_f64(value),
            DurationUnit::Seconds => value,
            DurationUnit::Milliseconds => value / 1000.0,
        })
        .map_err(|e| Error::custom(e))?
        .into())
    }
}

enum DurationUnit {
    Unspecified,
    Milliseconds,
    Seconds,
}

impl FromStr for DurationUnit {
    type Err = InvalidUnit;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "" => Ok(Self::Unspecified),
            "ms" => Ok(Self::Milliseconds),
            "s" => Ok(Self::Seconds),
            _ => Err(InvalidUnit(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, to_value};

    use super::CustomDuration;

    #[test]
    fn deserialize_data() -> Result<(), serde_json::Error> {
        let json_value = to_value("100")?;
        dbg!(serde_json::from_value::<CustomDuration>(json_value))
            .expect("is a valid duration expression");

        let json_value = to_value("100ms")?;
        dbg!(serde_json::from_value::<CustomDuration>(json_value))
            .expect("is a valid duration expression");

        let json_value = to_value(100)?;
        dbg!(serde_json::from_value::<CustomDuration>(json_value))
            .expect("is a valid duration expression");

        let json_value = to_value(100.0)?;
        dbg!(serde_json::from_value::<CustomDuration>(json_value))
            .expect("is a valid duration expression");

        Ok(())
    }

    #[test]
    fn deserialize_invalid_data() {
        let json_value = json!("100#$ms&");

        dbg!(serde_json::from_value::<CustomDuration>(json_value))
            .expect_err("not a valid duration expression");

        let json_value = json!("100ms&");
        dbg!(serde_json::from_value::<CustomDuration>(json_value))
            .expect_err("not a valid duration expression");
    }
}
