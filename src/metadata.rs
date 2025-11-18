use serde::{Deserialize, Serialize};

pub const MAGIC_BYTES: &[u8; 4] = b"ECTF";
pub const VERSION: u8 = 0x01;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub nonce: [u8; 12],
    pub salt: [u8; 32],
    pub helper_question: String,
    pub original_name: String,
    pub content_type: ContentType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    File,
    Folder,
}

impl Metadata {
    pub fn new(nonce: [u8; 12], salt: [u8; 32], helper_question: String, original_name: String, content_type: ContentType) -> Self {
        Self {
            nonce,
            salt,
            helper_question,
            original_name,
            content_type,
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    pub fn deserialize(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_serialization() {
        let nonce = [0u8; 12];
        let salt = [0u8; 32];
        let metadata = Metadata::new(
            nonce,
            salt,
            "What is your favorite color?".to_string(),
            "test.txt".to_string(),
            ContentType::File,
        );

        let serialized = metadata.serialize().unwrap();
        let deserialized = Metadata::deserialize(&serialized).unwrap();

        assert_eq!(metadata.helper_question, deserialized.helper_question);
        assert_eq!(metadata.original_name, deserialized.original_name);
        assert_eq!(metadata.nonce, deserialized.nonce);
        assert_eq!(metadata.salt, deserialized.salt);
        assert_eq!(format!("{:?}", metadata.content_type), format!("{:?}", deserialized.content_type));
    }
}

