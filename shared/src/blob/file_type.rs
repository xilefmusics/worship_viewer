use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};

#[cfg(feature = "backend")]
use utoipa::ToSchema;

/// Stored image format for blobs. Wire values are MIME type strings.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "backend", derive(ToSchema))]
pub enum FileType {
    PNG,
    JPEG,
    SVG,
}

impl FileType {
    pub fn file_ending(&self) -> &'static str {
        match self {
            Self::PNG => ".png",
            Self::JPEG => ".jpeg",
            Self::SVG => ".svg",
        }
    }

    pub fn mime(&self) -> &'static str {
        match self {
            Self::PNG => "image/png",
            Self::JPEG => "image/jpeg",
            Self::SVG => "image/svg+xml",
        }
    }
}

impl Serialize for FileType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.mime())
    }
}

impl<'de> Deserialize<'de> for FileType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FileTypeVisitor)
    }
}

struct FileTypeVisitor;

impl Visitor<'_> for FileTypeVisitor {
    type Value = FileType;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a supported image MIME type string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v {
            "image/png" => Ok(FileType::PNG),
            "image/jpeg" => Ok(FileType::JPEG),
            "image/svg+xml" => Ok(FileType::SVG),
            "image/svg" => {
                #[cfg(feature = "backend")]
                tracing::warn!("deprecated MIME image/svg for SVG blobs; prefer image/svg+xml");
                Ok(FileType::SVG)
            }
            _ => Err(E::custom(format!("unknown file type: {v}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FileType;

    #[test]
    fn serde_svg_serializes_as_registered_mime() {
        assert_eq!(
            serde_json::to_string(&FileType::SVG).unwrap(),
            "\"image/svg+xml\""
        );
    }

    #[test]
    fn deserializes_svg_xml_and_legacy_svg() {
        let a: FileType = serde_json::from_str("\"image/svg+xml\"").unwrap();
        assert_eq!(a, FileType::SVG);
        let b: FileType = serde_json::from_str("\"image/svg\"").unwrap();
        assert_eq!(b, FileType::SVG);
    }
}
