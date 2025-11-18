use serde::{Deserialize, Serialize};

pub const MAGIC_BYTES: &[u8; 4] = b"ECTF";
pub const VERSION: u8 = 0x01;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub nonce: [u8; 12],
    pub salt: [u8; 32],
    pub helper_question: String,
    pub original_filename: String,
}

impl Metadata {
    pub fn new(nonce: [u8; 12], salt: [u8; 32], helper_question: String, original_filename: String) -> Self {
        Self {
            nonce,
            salt,
            helper_question,
            original_filename,
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
        );

        let serialized = metadata.serialize().unwrap();
        let deserialized = Metadata::deserialize(&serialized).unwrap();

        assert_eq!(metadata.helper_question, deserialized.helper_question);
        assert_eq!(metadata.original_filename, deserialized.original_filename);
        assert_eq!(metadata.nonce, deserialized.nonce);
        assert_eq!(metadata.salt, deserialized.salt);
    }
}

