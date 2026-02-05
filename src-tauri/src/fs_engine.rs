use jwalk::WalkDir;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub modified: i64,
    pub permissions: String,
    pub owner: u32,
    pub group: u32,
    pub extension: Option<String>,
    pub hidden: bool,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryContents {
    pub path: String,
    pub parent: Option<String>,
    pub entries: Vec<FileEntry>,
    pub total_items: usize,
    pub total_size: u64,
}

#[cfg(unix)]
fn mode_to_string(mode: u32, is_dir: bool) -> String {
    let mut result = String::with_capacity(10);

    result.push(if is_dir { 'd' } else { '-' });

    result.push(if mode & 0o400 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o200 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o100 != 0 { 'x' } else { '-' });

    result.push(if mode & 0o040 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o020 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o010 != 0 { 'x' } else { '-' });

    result.push(if mode & 0o004 != 0 { 'r' } else { '-' });
    result.push(if mode & 0o002 != 0 { 'w' } else { '-' });
    result.push(if mode & 0o001 != 0 { 'x' } else { '-' });

    result
}

#[cfg(windows)]
fn mode_to_string(_mode: u32, is_dir: bool) -> String {
    if is_dir {
        "d---------".to_string()
    } else {
        "----------".to_string()
    }
}

fn get_file_entry(path: &Path) -> Option<FileEntry> {
    let metadata = fs::symlink_metadata(path).ok()?;
    let name = path.file_name()?.to_string_lossy().to_string();
    let is_symlink = metadata.is_symlink();

    let real_metadata = if is_symlink {
        fs::metadata(path).ok()
    } else {
        Some(metadata.clone())
    };

    let is_dir = real_metadata
        .as_ref()
        .map(|m| m.is_dir())
        .unwrap_or(false);

    let size = if is_dir {
        0
    } else {
        real_metadata
            .as_ref()
            .map(|m| m.len())
            .unwrap_or(metadata.len())
    };

    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    #[cfg(unix)]
    let mode = metadata.permissions().mode();
    #[cfg(windows)]
    let mode = 0;

    let permissions = mode_to_string(mode, is_dir);

    let extension = if is_dir {
        None
    } else {
        path.extension()
            .map(|e| e.to_string_lossy().to_lowercase())
    };

    let hidden = name.starts_with('.');

    #[cfg(unix)]
    let (owner, group) = (metadata.uid(), metadata.gid());
    #[cfg(windows)]
    let (owner, group) = (0, 0); // Windows uses different permission model

    Some(FileEntry {
        name,
        path: path.to_string_lossy().to_string(),
        is_dir,
        is_symlink,
        size,
        modified,
        permissions,
        owner,
        group,
        extension,
        hidden,
    })
}

pub fn read_directory(path: &str, show_hidden: bool) -> Result<DirectoryContents, String> {
    let dir_path = PathBuf::from(path);

    if !dir_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    if !dir_path.is_dir() {
        return Err(format!("Path is not a directory: {}", path));
    }

    let read_dir = fs::read_dir(&dir_path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    let paths: Vec<PathBuf> = read_dir
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .collect();

    let mut entries: Vec<FileEntry> = paths
        .par_iter()
        .filter_map(|path| get_file_entry(path))
        .filter(|entry| show_hidden || !entry.hidden)
        .collect();

    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    let total_items = entries.len();
    let total_size: u64 = entries.iter().map(|e| e.size).sum();

    let parent = dir_path.parent().map(|p| p.to_string_lossy().to_string());

    Ok(DirectoryContents {
        path: path.to_string(),
        parent,
        entries,
        total_items,
        total_size,
    })
}

pub fn crawl_directory(root: &str, max_depth: Option<usize>) -> Vec<FileEntry> {
    let walker = WalkDir::new(root)
        .skip_hidden(false)
        .max_depth(max_depth.unwrap_or(usize::MAX))
        .parallelism(jwalk::Parallelism::RayonNewPool(num_cpus::get()));

    walker
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| get_file_entry(&entry.path()))
        .collect()
}

pub fn get_file_preview(path: &str, max_bytes: usize) -> Result<FilePreview, String> {
    let file_path = PathBuf::from(path);

    if !file_path.exists() {
        return Err("File does not exist".to_string());
    }

    if file_path.is_dir() {
        return Err("Cannot preview directories".to_string());
    }

    let metadata = fs::metadata(&file_path)
        .map_err(|e| format!("Failed to read metadata: {}", e))?;

    let extension = file_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase());

    let preview_type = match extension.as_deref() {
        Some("rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "svelte" | "vue" |
             "html" | "css" | "scss" | "sass" | "json" | "yaml" | "yml" |
             "toml" | "xml" | "md" | "txt" | "sh" | "bash" | "zsh" |
             "c" | "cpp" | "h" | "hpp" | "go" | "java" | "kt" | "swift" |
             "rb" | "php" | "sql" | "lua" | "vim" | "conf" | "ini" |
             "dockerfile" | "makefile" | "cmake") => PreviewType::Code,
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" | "ico" | "bmp" | "tiff" | "avif") => PreviewType::Image,
        Some("exe" | "bin" | "so" | "dylib" | "dll" | "o" | "a") => PreviewType::Hex,
        _ => PreviewType::Auto,
    };

    let content = fs::read(&file_path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    let truncated = content.len() > max_bytes;
    let preview_bytes: Vec<u8> = content.into_iter().take(max_bytes).collect();

    let is_text = !preview_bytes.iter().take(8192).any(|&b| b == 0);

    let (final_type, text_content, hex_content) = match preview_type {
        PreviewType::Code => {
            let text = String::from_utf8_lossy(&preview_bytes).to_string();
            (PreviewType::Code, Some(text), None)
        }
        PreviewType::Image => (PreviewType::Image, None, None),
        PreviewType::Hex => {
            let hex = bytes_to_hex(&preview_bytes);
            (PreviewType::Hex, None, Some(hex))
        }
        PreviewType::Auto => {
            if is_text {
                let text = String::from_utf8_lossy(&preview_bytes).to_string();
                (PreviewType::Code, Some(text), None)
            } else {
                let hex = bytes_to_hex(&preview_bytes);
                (PreviewType::Hex, None, Some(hex))
            }
        }
    };

    Ok(FilePreview {
        path: path.to_string(),
        preview_type: final_type,
        size: metadata.len(),
        text_content,
        hex_content,
        truncated,
        extension,
    })
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut result = String::new();
    for (i, chunk) in bytes.chunks(16).enumerate() {
        result.push_str(&format!("{:08x}  ", i * 16));

        for (j, byte) in chunk.iter().enumerate() {
            if j == 8 {
                result.push(' ');
            }
            result.push_str(&format!("{:02x} ", byte));
        }

        for j in chunk.len()..16 {
            if j == 8 {
                result.push(' ');
            }
            result.push_str("   ");
        }

        result.push_str(" |");
        for byte in chunk {
            let c = if *byte >= 0x20 && *byte < 0x7f {
                *byte as char
            } else {
                '.'
            };
            result.push(c);
        }
        result.push_str("|\n");
    }
    result
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreviewType {
    Code,
    Image,
    Hex,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreview {
    pub path: String,
    pub preview_type: PreviewType,
    pub size: u64,
    pub text_content: Option<String>,
    pub hex_content: Option<String>,
    pub truncated: bool,
    pub extension: Option<String>,
}

pub fn get_home_dir() -> Option<String> {
    dirs::home_dir().map(|p| p.to_string_lossy().to_string())
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}
