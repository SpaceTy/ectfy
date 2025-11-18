mod archive;
mod cli;
mod encryption;
mod file_ops;
mod metadata;
mod password;
mod selection;

use std::io::{self, Write};
use std::path::Path;

use cli::Cli;
use file_ops::{decrypt_file, decrypt_folder_archive, encrypt_file, encrypt_folder_archive, is_encrypted_file, read_encrypted_file};
use metadata::ContentType;
use password::{get_password, get_password_with_confirmation};
use selection::select_files_interactive;

fn prompt_helper_question() -> io::Result<String> {
    print!("Enter helper question for decryption: ");
    io::stdout().flush()?;
    let mut question = String::new();
    io::stdin().read_line(&mut question)?;
    Ok(question.trim().to_string())
}

fn process_path(path: &Path, show_password: bool) -> Result<(), String> {
    if is_encrypted_file(path) {
        let (metadata, _) = read_encrypted_file(path)
            .map_err(|e| format!("Failed to read encrypted file: {}", e))?;
        
        println!("Helper question: {}", metadata.helper_question);
        
        let password = get_password(show_password)
            .map_err(|e| format!("Failed to read password: {}", e))?;

        match metadata.content_type {
            ContentType::File => {
                let decrypted = decrypt_file(path, &password)?;
                println!("✓ Decrypted {} → {}", path.display(), decrypted.display());
            }
            ContentType::Folder => {
                let decrypted = decrypt_folder_archive(path, &password)?;
                println!("✓ Decrypted {} → {}", path.display(), decrypted.display());
            }
        }
    } else {
        let password = get_password_with_confirmation(show_password)
            .map_err(|e| format!("Failed to read password: {}", e))?;

        let helper_question = prompt_helper_question()
            .map_err(|e| format!("Failed to read helper question: {}", e))?;

        if helper_question.is_empty() {
            return Err("Helper question cannot be empty".to_string());
        }

        if path.is_file() {
            let encrypted = encrypt_file(path, &password, &helper_question)?;
            println!("✓ Encrypted {} → {}", path.display(), encrypted.display());
        } else if path.is_dir() {
            let encrypted = encrypt_folder_archive(path, &password, &helper_question)?;
            println!("✓ Encrypted {} → {}", path.display(), encrypted.display());
        } else {
            return Err(format!("Path does not exist: {}", path.display()));
        }
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse_args();

    let result = if let Some(path) = cli.path {
        if !path.exists() {
            eprintln!("❌ Error: Path does not exist: {}", path.display());
            std::process::exit(1);
        }
        process_path(&path, cli.show_password)
    } else {
        let files = match select_files_interactive() {
            Ok(files) => files,
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                std::process::exit(1);
            }
        };

        if files.is_empty() {
            println!("No files selected");
            return;
        }

        let mut success_count = 0;
        let mut error_count = 0;

        for file_path_str in files {
            let file_path = Path::new(&file_path_str);
            if !file_path.exists() {
                eprintln!("❌ Error: Path does not exist: {}", file_path.display());
                error_count += 1;
                continue;
            }

            match process_path(file_path, cli.show_password) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    eprintln!("❌ Error processing {}: {}", file_path.display(), e);
                    error_count += 1;
                }
            }
        }

        if success_count > 0 {
            println!("\n✓ Successfully processed {} file(s)", success_count);
        }
        if error_count > 0 {
            eprintln!("❌ Failed to process {} file(s)", error_count);
            std::process::exit(1);
        }
        Ok(())
    };

    if let Err(e) = result {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

