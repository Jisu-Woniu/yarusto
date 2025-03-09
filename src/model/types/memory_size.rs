use std::{fmt, ops::Deref, result::Result, str::FromStr};

use serde::{
    de::{self, Error, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use size::{consts::KIBIBYTE, Size};

use crate::model::types::InvalidUnit;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CustomSize(pub(crate) Size);

impl CustomSize {
    pub const fn as_kibibyte(self) -> u32 {
        (self.0.bytes() / KIBIBYTE) as _
    }
}

impl Default for CustomSize {
    fn default() -> Self {
        Self(Size::from_mebibytes(128))
    }
}

impl Deref for CustomSize {
    type Target = Size;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialOrd for CustomSize {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for CustomSize {}

impl Ord for CustomSize {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes().cmp(&other.bytes())
    }
}

impl Serialize for CustomSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64((self.0.bytes() as u64) >> 10)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MemorySizeUnit {
    Unspecified,
    Kibibyte,
    Mebibyte,
    Gibibyte,
}

impl FromStr for MemorySizeUnit {
    type Err = InvalidUnit;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "" => Ok(Self::Unspecified),
            "kib" | "kb" => Ok(Self::Kibibyte),
            "mib" | "mb" => Ok(Self::Mebibyte),
            "gib" | "gb" => Ok(Self::Gibibyte),
            _ => Err(InvalidUnit(s.to_string())),
        }
    }
}

impl<'de> Deserialize<'de> for CustomSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CustomSizeVisitor;

        impl Visitor<'_> for CustomSizeVisitor {
            type Value = CustomSize;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a value representing memory size, between 64MiB and 10GiB (10,240MiB), in MiB or GiB")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    1..=10 => Ok(CustomSize(Size::from_gibibytes(v))),
                    64..=10240 => Ok(CustomSize(Size::from_mebibytes(v))),
                    _ => Err(Error::invalid_value(Unexpected::Unsigned(v), &self)),
                }
            }

            /// Deserialize f64, may cause
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v {
                    0.1..=10.0 => Ok(CustomSize(Size::from_gibibytes(v))),
                    64.0..=10240.0 => Ok(CustomSize(Size::from_mebibytes(v))),
                    _ => Err(Error::invalid_value(Unexpected::Float(v), &self)),
                }
            }
        }

        deserializer.deserialize_any(CustomSizeVisitor)
    }
}
