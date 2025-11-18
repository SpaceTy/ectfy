use std::process::{Command, Stdio};
use std::io::{self, Write};

#[allow(dead_code)]
pub fn select_files_with_fzf() -> Result<Vec<String>, String> {
    let fzf = Command::new("fzf")
        .arg("--multi")
        .arg("--print0")
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                "fzf is required but not installed. Please install fzf first.".to_string()
            } else {
                format!("Failed to spawn fzf: {}", e)
            }
        })?;

    let output = fzf.wait_with_output()
        .map_err(|e| format!("Failed to wait for fzf: {}", e))?;

    if !output.status.success() {
        return Err("File selection cancelled or fzf exited with error".to_string());
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 from fzf: {}", e))?;

    if stdout.is_empty() {
        return Ok(Vec::new());
    }

    let files: Vec<String> = stdout
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    Ok(files)
}

pub fn find_ect_files() -> Result<Vec<String>, String> {
    let find = Command::new("find")
        .arg(".")
        .arg("-name")
        .arg("*.ect")
        .arg("-type")
        .arg("f")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn find: {}", e))?;

    let output = find.wait_with_output()
        .map_err(|e| format!("Failed to wait for find: {}", e))?;

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 from find: {}", e))?;

    let files: Vec<String> = stdout
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    Ok(files)
}

pub fn find_all_files() -> Result<Vec<String>, String> {
    let find = Command::new("find")
        .arg(".")
        .arg("-type")
        .arg("f")
        .arg("!")
        .arg("-name")
        .arg("*.ect")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn find: {}", e))?;

    let output = find.wait_with_output()
        .map_err(|e| format!("Failed to wait for find: {}", e))?;

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 from find: {}", e))?;

    let files: Vec<String> = stdout
        .lines()
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    Ok(files)
}

pub fn select_files_interactive() -> Result<Vec<String>, String> {
    let all_files = find_all_files()?;
    let ect_files = find_ect_files()?;

    if all_files.is_empty() && ect_files.is_empty() {
        return Err("No files found in current directory".to_string());
    }

    let mut input = String::new();
    for file in &all_files {
        input.push_str(file);
        input.push('\n');
    }
    for file in &ect_files {
        input.push_str(file);
        input.push('\n');
    }

    let mut fzf = Command::new("fzf")
        .arg("--multi")
        .arg("--print0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                "fzf is required but not installed. Please install fzf first.".to_string()
            } else {
                format!("Failed to spawn fzf: {}", e)
            }
        })?;

    if let Some(mut stdin) = fzf.stdin.take() {
        stdin.write_all(input.as_bytes())
            .map_err(|e| format!("Failed to write to fzf stdin: {}", e))?;
    }

    let output = fzf.wait_with_output()
        .map_err(|e| format!("Failed to wait for fzf: {}", e))?;

    if !output.status.success() {
        return Err("File selection cancelled or fzf exited with error".to_string());
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| format!("Invalid UTF-8 from fzf: {}", e))?;

    if stdout.is_empty() {
        return Ok(Vec::new());
    }

    let files: Vec<String> = stdout
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| s.trim().to_string())
        .collect();

    Ok(files)
}

