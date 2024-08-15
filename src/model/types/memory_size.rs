use std::{fmt, result::Result, str::FromStr};

use serde::{
    de::{self, Error, Unexpected, Visitor},
    Deserialize, Serialize,
};

use super::InvalidUnit;

#[derive(Debug)]
pub struct MemorySize {
    value: u32,
    unit: MemorySizeUnit,
}

impl MemorySize {
    pub fn as_kib(&self) -> u32 {
        match self.unit {
            MemorySizeUnit::Kibibyte => self.value,
            MemorySizeUnit::Mebibyte => self.value << 10,
            MemorySizeUnit::Gibibyte => self.value << 20,
        }
    }
}

impl Default for MemorySize {
    fn default() -> Self {
        Self {
            value: 128,
            unit: MemorySizeUnit::Mebibyte,
        }
    }
}

#[derive(Debug, Default)]
pub enum MemorySizeUnit {
    Kibibyte,
    #[default]
    Mebibyte,
    Gibibyte,
}

impl FromStr for MemorySizeUnit {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "KiB" | "KB" | "kb" => Ok(Self::Kibibyte),
            "MiB" | "MB" | "mb" | "" => Ok(Self::Mebibyte),
            "GiB" | "GB" | "gb" => Ok(Self::Gibibyte),
            _ => Err(InvalidUnit(s.to_string())),
        }
    }

    type Err = InvalidUnit;
}

impl<'de> Deserialize<'de> for MemorySize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MemorySizeVisitor;

        impl<'de> Visitor<'de> for MemorySizeVisitor {
            type Value = MemorySize;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing memory size")
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let value = match v {
                    // SAFETY: v is neither `NaN` nor `Inf`, and is in the valid range of i32
                    1.0..=10240.0 => unsafe { v.to_int_unchecked() },
                    _ => {
                        return Err(Error::invalid_value(
                            Unexpected::Float(v),
                            &"a value between 1MiB and 10GiB (10,240MiB)",
                        ));
                    }
                };
                Ok(MemorySize {
                    value,
                    unit: MemorySizeUnit::Mebibyte,
                })
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let (value, unit) = value.split_at(
                    value
                        .find(|c: char| c.is_ascii_alphabetic())
                        .unwrap_or(value.len()),
                );
                let value = value.parse().map_err(|_| {
                    de::Error::invalid_value(de::Unexpected::Str(value), &"a valid integer")
                })?;
                let unit = MemorySizeUnit::from_str(unit).unwrap_or(MemorySizeUnit::Mebibyte);

                Ok(MemorySize { value, unit })
            }
        }

        deserializer.deserialize_any(MemorySizeVisitor)
    }
}

impl Serialize for MemorySize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.as_kib())
    }
}
