use serde::de::{Error as DeError, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(feature = "backend")]
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(
    feature = "backend",
    derive(ToSchema),
    schema(rename_all = "snake_case")
)]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

impl Serialize for Orientation {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.to_str())
    }
}

impl<'de> Deserialize<'de> for Orientation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(OrientationVisitor)
    }
}

struct OrientationVisitor;

impl Visitor<'_> for OrientationVisitor {
    type Value = Orientation;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("portrait or landscape (snake_case; PascalCase accepted)")
    }

    fn visit_str<E: DeError>(self, s: &str) -> Result<Self::Value, E> {
        Ok(match s {
            "portrait" => Orientation::Portrait,
            "Portrait" => {
                legacy_enum_warn("Orientation", s);
                Orientation::Portrait
            }
            "landscape" => Orientation::Landscape,
            "Landscape" => {
                legacy_enum_warn("Orientation", s);
                Orientation::Landscape
            }
            _ => return Err(E::custom(format!("unknown orientation: {s}"))),
        })
    }
}

pub(crate) fn legacy_enum_warn(enum_name: &str, value: &str) {
    #[cfg(feature = "backend")]
    tracing::warn!(
        enum_name,
        value,
        "legacy enum wire value; prefer lower_snake_case"
    );
    #[cfg(not(feature = "backend"))]
    let _ = (enum_name, value);
}

impl Orientation {
    pub fn from_dimensions(dimensions: (f64, f64)) -> Self {
        if dimensions.0 > dimensions.1 {
            Self::Landscape
        } else {
            Self::Portrait
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Portrait => "portrait",
            Self::Landscape => "landscape",
        }
    }
}
