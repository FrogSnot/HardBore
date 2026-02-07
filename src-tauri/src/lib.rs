pub mod fs_engine;
pub mod indexer;

use fs_engine::{read_directory, get_file_preview, get_home_dir, DirectoryContents, FilePreview};
use indexer::{Indexer, IndexerStatus, SearchResult};
use serde::Serialize;
use std::sync::Mutex;
use std::path::Path;
use std::process::Command;
use tauri::{Manager, State};

#[derive(Debug, Clone, Serialize)]
pub struct PickerConfig {
    pub mode: PickerMode,
    pub allow_multiple: bool,
    pub file_types: Option<Vec<String>>,
    pub start_dir: Option<String>,
    pub current_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum PickerMode {
    Disabled,
    Files,
    Directories,
    Both,
    Save,
}

struct AppState {
    indexer: Mutex<Option<Indexer>>,
    picker_config: Mutex<PickerConfig>,
}

#[tauri::command]
fn init_indexer(app_handle: tauri::AppHandle, state: State<AppState>) -> Result<(), String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| e.to_string())?;

    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create data directory: {}", e))?;

    let indexer = Indexer::new(&data_dir.to_string_lossy())
        .map_err(|e| format!("Failed to initialize indexer: {}", e))?;

    let mut state_indexer = state.indexer.lock().unwrap();
    *state_indexer = Some(indexer);

    Ok(())
}

#[tauri::command]
fn read_dir(path: String, show_hidden: bool) -> Result<DirectoryContents, String> {
    read_directory(&path, show_hidden)
}

#[tauri::command]
fn preview_file(path: String, max_bytes: Option<usize>) -> Result<FilePreview, String> {
    get_file_preview(&path, max_bytes.unwrap_or(65536))
}

#[tauri::command]
fn get_home() -> Option<String> {
    get_home_dir()
}

#[tauri::command]
fn get_current_dir() -> Option<String> {
    std::env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn start_indexing(path: String, max_depth: Option<usize>, state: State<AppState>) -> Result<(), String> {
    let indexer = state.indexer.lock().unwrap();
    if let Some(ref idx) = *indexer {
        idx.index_directory(&path, max_depth);
        Ok(())
    } else {
        Err("Indexer not initialized".to_string())
    }
}

#[tauri::command]
fn search_files(query: String, limit: Option<usize>, state: State<AppState>) -> Vec<SearchResult> {
    let indexer = state.indexer.lock().unwrap();
    if let Some(ref idx) = *indexer {
        let results = idx.search_fts(&query, limit.unwrap_or(50));
        if results.is_empty() {
            idx.search_fuzzy(&query, limit.unwrap_or(50))
        } else {
            results
        }
    } else {
        vec![]
    }
}

#[tauri::command]
fn get_indexer_status(state: State<AppState>) -> Option<IndexerStatus> {
    let indexer = state.indexer.lock().unwrap();
    indexer.as_ref().map(|idx| idx.get_status())
}

#[tauri::command]
fn get_indexed_count(state: State<AppState>) -> usize {
    let indexer = state.indexer.lock().unwrap();
    indexer.as_ref().map(|idx| idx.get_indexed_count()).unwrap_or(0)
}

#[tauri::command]
fn clear_index(state: State<AppState>) -> Result<(), String> {
    let indexer = state.indexer.lock().unwrap();
    if let Some(ref idx) = *indexer {
        idx.clear_index().map_err(|e| e.to_string())
    } else {
        Err("Indexer not initialized".to_string())
    }
}

#[tauri::command]
fn delete_path(path: String, is_dir: bool) -> Result<(), String> {
    if is_dir {
        std::fs::remove_dir_all(&path)
            .map_err(|e| format!("Failed to delete directory: {}", e))
    } else {
        std::fs::remove_file(&path)
            .map_err(|e| format!("Failed to delete file: {}", e))
    }
}

#[tauri::command]
fn copy_path(source: String, destination: String) -> Result<(), String> {
    use std::fs;
    use std::path::Path;
    
    let src = Path::new(&source);
    let dst = Path::new(&destination);
    
    if !src.exists() {
        return Err("Source does not exist".to_string());
    }
    
    if src.is_dir() {
        copy_dir_recursive(src, dst)
    } else {
        fs::copy(src, dst)
            .map(|_| ())
            .map_err(|e| format!("Failed to copy file: {}", e))
    }
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    use std::fs;
    
    if !dst.exists() {
        fs::create_dir_all(dst)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    for entry in fs::read_dir(src)
        .map_err(|e| format!("Failed to read directory: {}", e))? 
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
        }
    }
    
    Ok(())
}

#[tauri::command]
fn move_path(source: String, destination: String) -> Result<(), String> {
    use std::fs;
    use std::path::Path;
    
    let src = Path::new(&source);
    let dst = Path::new(&destination);
    
    if !src.exists() {
        return Err("Source does not exist".to_string());
    }
    
    if let Err(_) = fs::rename(src, dst) {
        if src.is_dir() {
            copy_dir_recursive(src, dst)?;
            fs::remove_dir_all(src)
                .map_err(|e| format!("Failed to remove source directory: {}", e))?;
        } else {
            fs::copy(src, dst)
                .map_err(|e| format!("Failed to copy file: {}", e))?;
            fs::remove_file(src)
                .map_err(|e| format!("Failed to remove source file: {}", e))?;
        }
    }
    
    Ok(())
}

#[tauri::command]
fn batch_copy_paths(sources: Vec<String>, destination_dir: String) -> Result<Vec<String>, String> {
    use std::path::Path;
    
    let dest_dir = Path::new(&destination_dir);
    if !dest_dir.is_dir() {
        return Err("Destination must be a directory".to_string());
    }
    
    let mut errors = Vec::new();
    
    for source in sources {
        let src_path = Path::new(&source);
        let file_name = src_path.file_name()
            .ok_or_else(|| format!("Invalid source path: {}", source))?;
        let dest_path = dest_dir.join(file_name);
        
        if let Err(e) = copy_path(source.clone(), dest_path.to_string_lossy().to_string()) {
            errors.push(format!("{}: {}", source, e));
        }
    }
    
    if errors.is_empty() {
        Ok(vec![])
    } else {
        Err(errors.join("\n"))
    }
}

#[tauri::command]
fn batch_move_paths(sources: Vec<String>, destination_dir: String) -> Result<Vec<String>, String> {
    use std::path::Path;
    
    let dest_dir = Path::new(&destination_dir);
    if !dest_dir.is_dir() {
        return Err("Destination must be a directory".to_string());
    }
    
    let mut errors = Vec::new();
    
    for source in sources {
        let src_path = Path::new(&source);
        let file_name = src_path.file_name()
            .ok_or_else(|| format!("Invalid source path: {}", source))?;
        let dest_path = dest_dir.join(file_name);
        
        if let Err(e) = move_path(source.clone(), dest_path.to_string_lossy().to_string()) {
            errors.push(format!("{}: {}", source, e));
        }
    }
    
    if errors.is_empty() {
        Ok(vec![])
    } else {
        Err(errors.join("\n"))
    }
}

#[tauri::command]
fn rename_path(old_path: String, new_name: String) -> Result<String, String> {
    let path = Path::new(&old_path);
    let parent = path.parent()
        .ok_or("Cannot get parent directory")?;
    let new_path = parent.join(&new_name);
    
    std::fs::rename(&old_path, &new_path)
        .map_err(|e| format!("Failed to rename: {}", e))?;
    
    Ok(new_path.to_string_lossy().to_string())
}

#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn show_in_folder(path: String) -> Result<(), String> {
    let target_path = Path::new(&path);
    let folder = if target_path.is_dir() {
        target_path
    } else {
        target_path.parent().ok_or("Cannot get parent directory")?
    };
    
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(folder)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-R")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg("/select,")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn open_terminal(path: String) -> Result<(), String> {
    let target_path = Path::new(&path);
    let folder = if target_path.is_dir() {
        target_path.to_path_buf()
    } else {
        target_path.parent()
            .ok_or("Cannot get parent directory")?
            .to_path_buf()
    };
    
    #[cfg(target_os = "linux")]
    {
        let terminals = ["kitty", "alacritty", "gnome-terminal", "konsole", "xterm"];
        for term in terminals.iter() {
            if Command::new("which")
                .arg(term)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                Command::new(term)
                    .arg(if *term == "gnome-terminal" { "--working-directory" } else { "-d" })
                    .arg(&folder)
                    .spawn()
                    .ok();
                return Ok(());
            }
        }
        Command::new("x-terminal-emulator")
            .current_dir(&folder)
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .arg(&folder)
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .arg("/c")
            .arg("start")
            .arg("cmd")
            .current_dir(&folder)
            .spawn()
            .map_err(|e| format!("Failed to open terminal: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
fn create_directory(path: String) -> Result<(), String> {
    std::fs::create_dir_all(&path)
        .map_err(|e| format!("Failed to create directory: {}", e))
}

#[tauri::command]
fn get_mount_points() -> Vec<MountPoint> {
    let mut mounts = Vec::new();
    
    #[cfg(target_os = "linux")]
    {
        use std::fs::File;
        use std::io::{BufRead, BufReader};
        
        if let Ok(file) = File::open("/proc/mounts") {
            let reader = BufReader::new(file);
            for line in reader.lines().flatten() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let device = parts[0];
                    let mount_point = parts[1];
                    let fs_type = parts[2];
                    
                    if !device.starts_with("/dev/") && !mount_point.starts_with("/media/") && !mount_point.starts_with("/mnt/") && mount_point != "/home" {
                        continue;
                    }
                    
                    if ["proc", "sysfs", "devtmpfs", "tmpfs", "securityfs", "cgroup", "pstore", "bpf", "devpts"].contains(&fs_type) {
                        continue;
                    }
                    
                    let name = if let Some(dev_name) = device.strip_prefix("/dev/") {
                        dev_name.to_string()
                    } else {
                        Path::new(mount_point).file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| mount_point.to_string())
                    };
                    
                    mounts.push(MountPoint {
                        name,
                        path: mount_point.to_string(),
                        device: device.to_string(),
                        fs_type: fs_type.to_string(),
                    });
                }
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = Command::new("df").arg("-H").output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for line in stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 6 {
                        let device = parts[0];
                        let mount_point = parts[8];
                        
                        if device.starts_with("/dev/") {
                            let name = Path::new(mount_point).file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| mount_point.to_string());
                            
                            mounts.push(MountPoint {
                                name,
                                path: mount_point.to_string(),
                                device: device.to_string(),
                                fs_type: String::new(),
                            });
                        }
                    }
                }
            }
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        for letter in b'A'..=b'Z' {
            let drive = format!("{}:\\", letter as char);
            if Path::new(&drive).exists() {
                mounts.push(MountPoint {
                    name: format!("Drive {}", letter as char),
                    path: drive.clone(),
                    device: drive,
                    fs_type: String::new(),
                });
            }
        }
    }
    
    mounts
}

#[tauri::command]
fn add_favorite(path: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| e.to_string())?;
    
    let favorites_file = data_dir.join("favorites.json");
    
    let mut favorites: Vec<String> = if favorites_file.exists() {
        let content = std::fs::read_to_string(&favorites_file)
            .map_err(|e| format!("Failed to read favorites: {}", e))?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    if !favorites.contains(&path) {
        favorites.push(path);
        let json = serde_json::to_string_pretty(&favorites)
            .map_err(|e| format!("Failed to serialize favorites: {}", e))?;
        std::fs::write(&favorites_file, json)
            .map_err(|e| format!("Failed to write favorites: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
fn remove_favorite(path: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    let data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e: tauri::Error| e.to_string())?;
    
    let favorites_file = data_dir.join("favorites.json");
    
    if favorites_file.exists() {
        let content = std::fs::read_to_string(&favorites_file)
            .map_err(|e| format!("Failed to read favorites: {}", e))?;
        let mut favorites: Vec<String> = serde_json::from_str(&content).unwrap_or_default();
        
        favorites.retain(|f| f != &path);
        
        let json = serde_json::to_string_pretty(&favorites)
            .map_err(|e| format!("Failed to serialize favorites: {}", e))?;
        std::fs::write(&favorites_file, json)
            .map_err(|e| format!("Failed to write favorites: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
fn get_favorites(app_handle: tauri::AppHandle) -> Vec<String> {
    let data_dir = match app_handle.path().app_data_dir() {
        Ok(dir) => dir,
        Err(_) => return Vec::new(),
    };
    
    let favorites_file = data_dir.join("favorites.json");
    
    if favorites_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&favorites_file) {
            return serde_json::from_str(&content).unwrap_or_default();
        }
    }
    
    Vec::new()
}

#[tauri::command]
fn get_picker_config(state: State<AppState>) -> PickerConfig {
    state.picker_config.lock().unwrap().clone()
}

#[tauri::command]
fn select_files(paths: Vec<String>, app_handle: tauri::AppHandle) -> Result<(), String> {
    for path in &paths {
        println!("HARDBORE_SELECTED:{}", path);
    }
    
    app_handle.exit(0);
    Ok(())
}

#[tauri::command]
fn cancel_picker(app_handle: tauri::AppHandle) -> Result<(), String> {
    println!("HARDBORE_CANCELLED");
    app_handle.exit(1);
    Ok(())
}

#[tauri::command]
fn get_properties(path: String) -> Result<FileProperties, String> {
    let metadata = std::fs::metadata(&path)
        .map_err(|e| format!("Failed to get metadata: {}", e))?;
    
    let path_obj = Path::new(&path);
    let name = path_obj.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    
    Ok(FileProperties {
        name,
        path: path.clone(),
        size: metadata.len(),
        is_dir: metadata.is_dir(),
        is_symlink: metadata.is_symlink(),
        readonly: metadata.permissions().readonly(),
        created: metadata.created()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs()),
        modified: metadata.modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs()),
        accessed: metadata.accessed()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs()),
    })
}

#[derive(serde::Serialize)]
struct FileProperties {
    name: String,
    path: String,
    size: u64,
    is_dir: bool,
    is_symlink: bool,
    readonly: bool,
    created: Option<u64>,
    modified: Option<u64>,
    accessed: Option<u64>,
}

#[derive(serde::Serialize)]
struct MountPoint {
    name: String,
    path: String,
    device: String,
    fs_type: String,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let args: Vec<String> = std::env::args().collect();
    let mut picker_mode = PickerMode::Disabled;
    let mut allow_multiple = false;
    let mut file_types: Option<Vec<String>> = None;
    let mut start_dir: Option<String> = None;
    let mut current_name: Option<String> = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--picker" => picker_mode = PickerMode::Files,
            "--picker-dirs" => picker_mode = PickerMode::Directories,
            "--picker-both" => picker_mode = PickerMode::Both,
            "--picker-save" => picker_mode = PickerMode::Save,
            "--multiple" => allow_multiple = true,
            "--types" => {
                if i + 1 < args.len() {
                    file_types = Some(args[i + 1].split(',').map(|s| s.to_string()).collect());
                    i += 1;
                }
            }
            "--start-dir" => {
                if i + 1 < args.len() {
                    start_dir = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--current-name" => {
                if i + 1 < args.len() {
                    current_name = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    let picker_config = PickerConfig {
        mode: picker_mode,
        allow_multiple,
        file_types,
        start_dir,
        current_name,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState {
            indexer: Mutex::new(None),
            picker_config: Mutex::new(picker_config),
        })
        .invoke_handler(tauri::generate_handler![
            init_indexer,
            read_dir,
            preview_file,
            get_home,
            get_current_dir,
            start_indexing,
            search_files,
            get_indexer_status,
            get_indexed_count,
            clear_index,
            delete_path,
            copy_path,
            move_path,
            batch_copy_paths,
            batch_move_paths,
            rename_path,
            open_path,
            show_in_folder,
            open_terminal,
            get_properties,
            create_directory,
            get_mount_points,
            add_favorite,
            remove_favorite,
            get_favorites,
            get_picker_config,
            select_files,
            cancel_picker,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
