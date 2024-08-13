use std::{fmt, ops::Deref, str::FromStr, time::Duration};

use serde::{
    de::{Error, Unexpected, Visitor},
    Deserialize, Serialize,
};

use crate::model::types::InvalidUnit;

#[derive(Debug, Serialize)]
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

impl<'de> Visitor<'de> for DurationVisitor {
    type Value = CustomDuration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string representing duration")
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(Duration::from_secs_f64(match v {
            0.1..10.0 => Ok(v),
            100.0..10000.0 => Ok(v / 1000.0),
            _ => Err(Error::invalid_value(
                Unexpected::Float(v),
                &"a value represents duration, ranges from 0.1s to 10s, or 100ms to 10,000ms",
            ))?,
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
            DurationUnit::Seconds => value,
            DurationUnit::Milliseconds => value / 1000.0,
        })
        .map_err(|e| Error::custom(e))?
        .into())
    }
}

enum DurationUnit {
    Milliseconds,
    Seconds,
}

impl FromStr for DurationUnit {
    type Err = InvalidUnit;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ms" | "" => Ok(Self::Milliseconds),
            "s" => Ok(Self::Seconds),
            _ => Err(InvalidUnit(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::CustomDuration;

    #[test]
    fn deserialize_invalid_data() {
        use serde_json::json;

        let json_value = json!("10#$ms&");

        dbg!(serde_json::from_value::<CustomDuration>(json_value))
            .expect_err("not a valid duration expression");

        let json_value = json!("10ms&");
        dbg!(serde_json::from_value::<CustomDuration>(json_value))
            .expect_err("not a valid duration expression");

        let json_value = json!("10ms");
        dbg!(serde_json::from_value::<CustomDuration>(json_value))
            .expect("is a valid duration expression");
    }
}
