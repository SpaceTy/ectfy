use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::encryption::{decrypt_data, encrypt_data, generate_salt};
use crate::metadata::{Metadata, MAGIC_BYTES, VERSION};

pub fn is_encrypted_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "ect")
        .unwrap_or(false)
}

pub fn read_encrypted_file(path: &Path) -> Result<(Metadata, Vec<u8>), String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    if buffer.len() < 9 {
        return Err("File appears to be corrupted (too short)".to_string());
    }

    let magic = &buffer[0..4];
    if magic != MAGIC_BYTES {
        return Err("File appears to be corrupted (invalid magic bytes)".to_string());
    }

    let version = buffer[4];
    if version != VERSION {
        return Err(format!("Unsupported file version: {}", version));
    }

    let metadata_len = u32::from_le_bytes([
        buffer[5], buffer[6], buffer[7], buffer[8]
    ]) as usize;

    if buffer.len() < 9 + metadata_len {
        return Err("File appears to be corrupted (metadata length invalid)".to_string());
    }

    let metadata_bytes = &buffer[9..9 + metadata_len];
    let metadata = Metadata::deserialize(metadata_bytes)
        .map_err(|e| format!("Failed to deserialize metadata: {}", e))?;

    let encrypted_data = &buffer[9 + metadata_len..];

    Ok((metadata, encrypted_data.to_vec()))
}

pub fn write_encrypted_file(path: &Path, metadata: &Metadata, encrypted_data: &[u8]) -> Result<(), String> {
    let metadata_bytes = metadata.serialize()
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

    let mut file = fs::File::create(path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    file.write_all(MAGIC_BYTES)
        .map_err(|e| format!("Failed to write magic bytes: {}", e))?;
    file.write_all(&[VERSION])
        .map_err(|e| format!("Failed to write version: {}", e))?;
    
    let metadata_len = metadata_bytes.len() as u32;
    file.write_all(&metadata_len.to_le_bytes())
        .map_err(|e| format!("Failed to write metadata length: {}", e))?;
    
    file.write_all(&metadata_bytes)
        .map_err(|e| format!("Failed to write metadata: {}", e))?;
    file.write_all(encrypted_data)
        .map_err(|e| format!("Failed to write encrypted data: {}", e))?;

    Ok(())
}

pub fn encrypt_file(path: &Path, password: &str, helper_question: &str) -> Result<PathBuf, String> {
    let data = fs::read(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let salt = generate_salt();
    let (encrypted_data, nonce) = encrypt_data(&data, password, &salt)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let original_filename = path.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "Invalid filename".to_string())?
        .to_string();

    let metadata = Metadata::new(nonce, salt, helper_question.to_string(), original_filename);

    let output_path = path.with_extension("ect");
    write_encrypted_file(&output_path, &metadata, &encrypted_data)?;

    fs::remove_file(path)
        .map_err(|e| format!("Failed to delete original file: {}", e))?;

    Ok(output_path)
}

pub fn decrypt_file(path: &Path, password: &str) -> Result<PathBuf, String> {
    let (metadata, encrypted_data) = read_encrypted_file(path)?;

    let decrypted_data = decrypt_data(&encrypted_data, password, &metadata.salt, &metadata.nonce)
        .map_err(|_| "Incorrect password or corrupted file".to_string())?;

    let output_path = path.parent()
        .ok_or_else(|| "Invalid file path".to_string())?
        .join(&metadata.original_filename);

    fs::write(&output_path, decrypted_data)
        .map_err(|e| format!("Failed to write decrypted file: {}", e))?;

    fs::remove_file(path)
        .map_err(|e| format!("Failed to delete encrypted file: {}", e))?;

    Ok(output_path)
}

pub fn collect_files_recursive(path: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    if path.is_file() {
        files.push(path.to_path_buf());
    } else if path.is_dir() {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }
    } else {
        return Err(format!("Path does not exist: {}", path.display()));
    }

    Ok(files)
}

pub fn encrypt_folder(path: &Path, password: &str, helper_question: &str) -> Result<Vec<PathBuf>, String> {
    let files = collect_files_recursive(path)?;
    let mut encrypted_files = Vec::new();

    for file in files {
        let encrypted = encrypt_file(&file, password, helper_question)?;
        encrypted_files.push(encrypted);
    }

    Ok(encrypted_files)
}

pub fn decrypt_folder(path: &Path, password: &str) -> Result<Vec<PathBuf>, String> {
    let files = collect_files_recursive(path)?;
    let mut decrypted_files = Vec::new();

    for file in files {
        if is_encrypted_file(&file) {
            let decrypted = decrypt_file(&file, password)?;
            decrypted_files.push(decrypted);
        }
    }

    Ok(decrypted_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_is_encrypted_file() {
        assert!(is_encrypted_file(Path::new("test.ect")));
        assert!(!is_encrypted_file(Path::new("test.txt")));
    }

    #[test]
    fn test_encrypt_decrypt_file_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, b"Hello, World!").unwrap();

        let password = "test_password";
        let helper_question = "What is your favorite color?";

        let encrypted_path = encrypt_file(&test_file, password, helper_question).unwrap();
        assert!(encrypted_path.exists());
        assert!(is_encrypted_file(&encrypted_path));
        assert!(!test_file.exists(), "Original file should be deleted after encryption");

        let decrypted_path = decrypt_file(&encrypted_path, password).unwrap();
        assert!(decrypted_path.exists());
        assert!(!encrypted_path.exists(), "Encrypted file should be deleted after decryption");

        let content = fs::read(&decrypted_path).unwrap();
        assert_eq!(content, b"Hello, World!");
    }

    #[test]
    fn test_encrypt_decrypt_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("empty.txt");
        fs::write(&test_file, b"").unwrap();

        let password = "test_password";
        let helper_question = "Test question";

        let encrypted_path = encrypt_file(&test_file, password, helper_question).unwrap();
        let decrypted_path = decrypt_file(&encrypted_path, password).unwrap();

        let content = fs::read(&decrypted_path).unwrap();
        assert_eq!(content, b"");
    }

    #[test]
    fn test_encrypt_decrypt_binary_file() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("binary.bin");
        let binary_data = vec![0u8, 1u8, 2u8, 255u8, 128u8, 64u8];
        fs::write(&test_file, &binary_data).unwrap();

        let password = "test_password";
        let helper_question = "Test question";

        let encrypted_path = encrypt_file(&test_file, password, helper_question).unwrap();
        let decrypted_path = decrypt_file(&encrypted_path, password).unwrap();

        let content = fs::read(&decrypted_path).unwrap();
        assert_eq!(content, binary_data);
    }

    #[test]
    fn test_encrypt_decrypt_special_characters_filename() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test file with spaces.txt");
        fs::write(&test_file, b"Content").unwrap();

        let password = "test_password";
        let helper_question = "Test question";

        let encrypted_path = encrypt_file(&test_file, password, helper_question).unwrap();
        let decrypted_path = decrypt_file(&encrypted_path, password).unwrap();

        assert!(decrypted_path.file_name().unwrap().to_str().unwrap().contains("test file with spaces"));
        let content = fs::read(&decrypted_path).unwrap();
        assert_eq!(content, b"Content");
    }

    #[test]
    fn test_encrypt_decrypt_folder() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let file1 = temp_dir.path().join("file1.txt");
        let file2 = subdir.join("file2.txt");
        fs::write(&file1, b"File 1 content").unwrap();
        fs::write(&file2, b"File 2 content").unwrap();

        let password = "test_password";
        let helper_question = "Test question";

        let encrypted_files = encrypt_folder(temp_dir.path(), password, helper_question).unwrap();
        assert_eq!(encrypted_files.len(), 2);
        assert!(!file1.exists(), "Original file1 should be deleted after encryption");
        assert!(!file2.exists(), "Original file2 should be deleted after encryption");

        for encrypted_file in &encrypted_files {
            assert!(is_encrypted_file(encrypted_file));
        }

        let decrypted_files = decrypt_folder(temp_dir.path(), password).unwrap();
        assert_eq!(decrypted_files.len(), 2);
        for encrypted_file in &encrypted_files {
            assert!(!encrypted_file.exists(), "Encrypted files should be deleted after decryption");
        }
    }

    #[test]
    fn test_wrong_password_fails() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, b"Secret content").unwrap();

        let password = "correct_password";
        let wrong_password = "wrong_password";
        let helper_question = "Test question";

        let encrypted_path = encrypt_file(&test_file, password, helper_question).unwrap();
        let result = decrypt_file(&encrypted_path, wrong_password);

        assert!(result.is_err());
    }
}

