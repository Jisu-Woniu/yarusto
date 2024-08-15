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
            // TODO: Use different types for parsed value and internal representation.
            MemorySizeUnit::Unspecified | MemorySizeUnit::Kibibyte => self.value,
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

#[derive(Debug)]
pub enum MemorySizeUnit {
    Unspecified,
    Kibibyte,
    Mebibyte,
    Gibibyte,
}

impl FromStr for MemorySizeUnit {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "" => Ok(Self::Unspecified),
            "kib" | "kb" => Ok(Self::Kibibyte),
            "mib" | "mb" => Ok(Self::Mebibyte),
            "gib" | "gb" => Ok(Self::Gibibyte),
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

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    1..=10 => {
                        let v = v * 1024;

                        Ok(MemorySize {
                            // SAFETY: v is neither `NaN` nor `Inf`, and is in the valid range of i32
                            value: v as _,
                            unit: MemorySizeUnit::Mebibyte,
                        })
                    }
                    64..=10240 => Ok(MemorySize {
                        // SAFETY: v is neither `NaN` nor `Inf`, and is in the valid range of i32
                        value: v as _,
                        unit: MemorySizeUnit::Mebibyte,
                    }),
                    _ => Err(Error::invalid_value(
                        Unexpected::Unsigned(v),
                        &"a value between 64MiB and 10GiB (10,240MiB), in MiB or GiB",
                    )),
                }
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    0.1..=10.0 => {
                        let v = v * 1024.0;

                        Ok(MemorySize {
                            // SAFETY: v is neither `NaN` nor `Inf`, and is in the valid range of i32
                            value: unsafe { v.to_int_unchecked() },
                            unit: MemorySizeUnit::Mebibyte,
                        })
                    }
                    64.0..=10240.0 => Ok(MemorySize {
                        // SAFETY: v is neither `NaN` nor `Inf`, and is in the valid range of i32
                        value: unsafe { v.to_int_unchecked() },
                        unit: MemorySizeUnit::Mebibyte,
                    }),
                    _ => Err(Error::invalid_value(
                        Unexpected::Float(v),
                        &"a value between 64MiB and 10GiB (10,240MiB), in MiB or GiB",
                    )),
                }
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

                let unit = MemorySizeUnit::from_str(unit).map_err(|e| Error::custom(e))?;

                if let MemorySizeUnit::Unspecified = unit {
                    // Map to `Self::visit_u64`
                    return self.visit_u32(value);
                }

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
