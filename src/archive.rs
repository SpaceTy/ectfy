use std::io::Cursor;
use std::path::Path;
use tar::{Builder, Archive};
use walkdir::WalkDir;

pub fn create_tar_archive(folder_path: &Path) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    {
        let mut builder = Builder::new(&mut buffer);

        for entry in WalkDir::new(folder_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let file_type = entry.file_type();

            if file_type.is_file() {
                let relative_path = path.strip_prefix(folder_path)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?;
                
                builder.append_path_with_name(path, relative_path)
                    .map_err(|e| format!("Failed to append file to tar: {}", e))?;
            } else if file_type.is_dir() {
                let relative_path = path.strip_prefix(folder_path)
                    .map_err(|e| format!("Failed to get relative path: {}", e))?;
                
                if relative_path != Path::new("") {
                    builder.append_path_with_name(path, relative_path)
                        .map_err(|e| format!("Failed to append directory to tar: {}", e))?;
                }
            }
        }

        builder.finish()
            .map_err(|e| format!("Failed to finish tar archive: {}", e))?;
    }

    Ok(buffer)
}

pub fn extract_tar_archive(archive_data: &[u8], extract_to: &Path) -> Result<(), String> {
    let cursor = Cursor::new(archive_data);
    let mut archive = Archive::new(cursor);
    
    archive.unpack(extract_to)
        .map_err(|e| format!("Failed to extract tar archive: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_create_and_extract_tar_archive() {
        let temp_dir = TempDir::new().unwrap();
        let test_folder = temp_dir.path().join("test_folder");
        fs::create_dir(&test_folder).unwrap();

        let subdir = test_folder.join("subdir");
        fs::create_dir(&subdir).unwrap();

        let file1 = test_folder.join("file1.txt");
        let file2 = subdir.join("file2.txt");
        fs::write(&file1, b"File 1 content").unwrap();
        fs::write(&file2, b"File 2 content").unwrap();

        let archive_data = create_tar_archive(&test_folder).unwrap();
        assert!(!archive_data.is_empty());

        let extract_dir = temp_dir.path().join("extracted");
        fs::create_dir(&extract_dir).unwrap();
        extract_tar_archive(&archive_data, &extract_dir).unwrap();

        let extracted_file1 = extract_dir.join("file1.txt");
        let extracted_file2 = extract_dir.join("subdir").join("file2.txt");
        
        assert!(extracted_file1.exists());
        assert!(extracted_file2.exists());
        
        assert_eq!(fs::read(&extracted_file1).unwrap(), b"File 1 content");
        assert_eq!(fs::read(&extracted_file2).unwrap(), b"File 2 content");
    }

    #[test]
    fn test_create_tar_empty_folder() {
        let temp_dir = TempDir::new().unwrap();
        let empty_folder = temp_dir.path().join("empty_folder");
        fs::create_dir(&empty_folder).unwrap();

        let archive_data = create_tar_archive(&empty_folder).unwrap();
        assert!(!archive_data.is_empty());
    }
}

