# Folder Encryption Implementation Plan

## Overview
Implement folder encryption that creates a single encrypted archive file (.ect) preserving the entire folder structure using TAR format.

## Key Requirements
- Create single .ect file per folder (named after the folder)
- Use TAR format for archiving (then encrypt the TAR)
- Preserve directory structure and file paths
- Extract to folder with original name for perfect round-trip
- No backward compatibility needed

## Implementation Steps

### 1. Add Dependencies
Update `Cargo.toml` to include TAR crate:
```toml
tar = "0.4"
```

### 2. Modify Metadata Structure
Update `src/metadata.rs` to distinguish between file and folder encryption:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub nonce: [u8; 12],
    pub salt: [u8; 32],
    pub helper_question: String,
    pub original_name: String,  // Renamed from original_filename
    pub content_type: ContentType,  // New: File or Folder
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    File,
    Folder,
}
```

### 3. Create Archive Module
New file `src/archive.rs` for TAR operations:

```rust
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tar::Builder;
use tar::Archive;

pub fn create_tar_archive(folder_path: &Path) -> Result<Vec<u8>, String> {
    // Create TAR archive in memory
    // Include all files with relative paths
}

pub fn extract_tar_archive(archive_data: &[u8], extract_to: &Path) -> Result<(), String> {
    // Extract TAR archive to specified path
}
```

### 4. Update File Operations
Modify `src/file_ops.rs`:

#### New Functions:
```rust
pub fn encrypt_folder_archive(path: &Path, password: &str, helper_question: &str) -> Result<PathBuf, String> {
    // 1. Create TAR archive of folder
    // 2. Encrypt TAR data
    // 3. Save as single .ect file
    // 4. Remove original folder
}

pub fn decrypt_folder_archive(path: &Path, password: &str) -> Result<PathBuf, String> {
    // 1. Decrypt .ect file
    // 2. Extract TAR archive
    // 3. Remove .ect file
}
```

#### Modified Functions:
- Update `encrypt_file` and `decrypt_file` to use `original_name` instead of `original_filename`
- Update `encrypt_folder` and `decrypt_folder` to call new archive functions

### 5. Update Main Logic
Modify `src/main.rs`:

```rust
fn process_path(path: &Path, show_password: bool) -> Result<(), String> {
    if is_encrypted_file(path) {
        // ... existing logic ...
        let content_type = determine_content_type(&metadata);
        match content_type {
            ContentType::File => {
                let decrypted = decrypt_file(path, &password)?;
                // ... existing file logic ...
            }
            ContentType::Folder => {
                let decrypted = decrypt_folder_archive(path, &password)?;
                // ... folder logic ...
            }
        }
    } else {
        // ... existing logic ...
        if path.is_file() {
            let encrypted = encrypt_file(path, &password, &helper_question)?;
            // ... existing file logic ...
        } else if path.is_dir() {
            let encrypted = encrypt_folder_archive(path, &password, &helper_question)?;
            // ... folder logic ...
        }
    }
}
```

### 6. Helper Functions
Add utility functions:

```rust
fn determine_content_type(metadata: &Metadata) -> ContentType {
    metadata.content_type.clone()
}

fn get_folder_name_from_path(path: &Path) -> String {
    // Extract folder name for .ect file naming
}
```

## File Structure Changes

```
src/
├── main.rs          # Updated to handle folder archives
├── cli.rs           # No changes needed
├── encryption.rs    # No changes needed
├── file_ops.rs      # Major updates for archive functions
├── metadata.rs      # Updated with ContentType enum
├── archive.rs       # NEW: TAR archive operations
├── password.rs      # No changes needed
└── selection.rs     # No changes needed
```

## Testing Strategy

### Unit Tests:
1. **Archive creation/extraction**: Test TAR operations with various folder structures
2. **Folder encryption/decryption**: Test complete round-trip
3. **Edge cases**: Empty folders, nested folders, special characters in names
4. **Large folders**: Performance testing with many files

### Integration Tests:
1. **CLI folder encryption**: Test command-line interface
2. **Mixed operations**: Encrypt folder, then encrypt individual files within
3. **Error handling**: Invalid passwords, corrupted archives

## Implementation Order

1. **Phase 1**: Update metadata structure and add ContentType enum
2. **Phase 2**: Create archive module with TAR operations
3. **Phase 3**: Implement folder archive encryption/decryption functions
4. **Phase 4**: Update main logic to use new functions
5. **Phase 5**: Add comprehensive tests
6. **Phase 6**: Test edge cases and performance

## Security Considerations

1. **Metadata encryption**: Since we're using TAR, file paths are encrypted within the TAR data
2. **No metadata leakage**: Folder structure is not visible without decryption
3. **Atomic operations**: Ensure folder encryption/decryption is atomic (all or nothing)
4. **Secure deletion**: Properly remove original folder after encryption

## Performance Considerations

1. **Memory usage**: For large folders, consider streaming TAR creation
2. **Progress indication**: Add progress bars for large folder operations
3. **Temporary files**: Use temp files for large archives to avoid memory issues

## Error Handling

1. **Corrupted archives**: Detect and handle corrupted TAR data
2. **Partial extraction**: Handle cases where extraction fails mid-way
3. **Disk space**: Check available space before extraction
4. **Permission errors**: Handle file permission issues gracefully