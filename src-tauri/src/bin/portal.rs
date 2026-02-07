use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use zbus::{interface, ConnectionBuilder};
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileFilter {
    name: String,
    patterns: Vec<String>,
}

pub struct FileChooserPortal {
    hardbore_path: String,
}

fn encode_file_uri(path: &str) -> String {
    let encoded: String = path
        .split('/')
        .map(|seg| {
            seg.bytes()
                .map(|b| match b {
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
                    | b'-' | b'_' | b'.' | b'~' => {
                        (b as char).to_string()
                    }
                    _ => format!("%{:02X}", b),
                })
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("/");
    format!("file://{}", encoded)
}

fn extract_bool(options: &HashMap<String, OwnedValue>, key: &str) -> bool {
    options
        .get(key)
        .and_then(|v| v.downcast_ref::<bool>().ok())
        .unwrap_or(false)
}

fn extract_current_folder(options: &HashMap<String, OwnedValue>) -> Option<String> {
    let val = options.get("current_folder")?;
    if let Ok(s) = val.downcast_ref::<String>() {
        if !s.is_empty() {
            return Some(s);
        }
    }
    if let Ok(json) = serde_json::to_value(val) {
        if let Some(arr) = json.as_array() {
            let bytes: Vec<u8> = arr.iter()
                .filter_map(|b| b.as_u64().map(|n| n as u8))
                .collect();
            let clean = bytes.split(|&b| b == 0).next().unwrap_or(&bytes);
            if !clean.is_empty() {
                return String::from_utf8(clean.to_vec()).ok();
            }
        }
        if let Some(s) = json.as_str() {
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn extract_current_name(options: &HashMap<String, OwnedValue>) -> Option<String> {
    let val = options.get("current_name")?;
    val.downcast_ref::<String>().ok().filter(|s| !s.is_empty())
}

fn extract_filenames(options: &HashMap<String, OwnedValue>) -> Vec<String> {
    let Some(val) = options.get("files") else {
        return vec![];
    };
    match serde_json::to_value(val) {
        Ok(json) => {
            json.as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            if let Some(s) = item.as_str() {
                                return Some(s.to_string());
                            }
                            if let Some(bytes) = item.as_array() {
                                let v: Vec<u8> = bytes
                                    .iter()
                                    .filter_map(|b| b.as_u64().map(|n| n as u8))
                                    .collect();
                                let clean = v.split(|&b| b == 0).next().unwrap_or(&v);
                                return String::from_utf8(clean.to_vec()).ok();
                            }
                            None
                        })
                        .collect()
                })
                .unwrap_or_default()
        }
        Err(e) => {
            eprintln!("[HardBore Portal] Failed to parse filenames: {}", e);
            vec![]
        }
    }
}

fn parse_filters(options: &HashMap<String, OwnedValue>) -> Vec<FileFilter> {
    let Some(val) = options.get("filters") else {
        return vec![];
    };

    let json = match serde_json::to_value(val) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("[HardBore Portal] Could not serialize filters: {}", e);
            return vec![];
        }
    };

    eprintln!("[HardBore Portal] Filters as JSON: {}", json);

    let Some(filters_arr) = json.as_array() else {
        return vec![];
    };

    let mut result = vec![];
    for filter_item in filters_arr {
        let Some(tuple) = filter_item.as_array() else { continue };
        if tuple.len() < 2 {
            continue;
        }
        let name = tuple[0].as_str().unwrap_or("Filter").to_string();
        let mut patterns = vec![];

        if let Some(pats) = tuple[1].as_array() {
            for pat in pats {
                let Some(pat_tuple) = pat.as_array() else { continue };
                if pat_tuple.len() < 2 {
                    continue;
                }
                let match_type = pat_tuple[0].as_u64().unwrap_or(99);
                let pattern = pat_tuple[1].as_str().unwrap_or("");
                if match_type == 0 && !pattern.is_empty() {
                    patterns.push(pattern.to_string());
                }
            }
        }

        if !patterns.is_empty() {
            result.push(FileFilter { name, patterns });
        }
    }

    result
}

impl FileChooserPortal {
    pub fn new() -> Self {
        let hardbore_path = if std::path::Path::new("/usr/local/bin/hardbore").exists() {
            "/usr/local/bin/hardbore".to_string()
        } else if std::path::Path::new("/usr/bin/hardbore").exists() {
            "/usr/bin/hardbore".to_string()
        } else if let Ok(output) = std::process::Command::new("which").arg("hardbore").output() {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                "hardbore".to_string()
            }
        } else {
            "hardbore".to_string()
        };

        Self { hardbore_path }
    }

    async fn launch_picker(&self, args: &[String]) -> Vec<String> {
        eprintln!("[HardBore Portal] Launching: {} {:?}", &self.hardbore_path, args);

        let output = Command::new(&self.hardbore_path)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(output) if output.status.success() => {
                eprintln!("[HardBore Portal] Picker exited successfully");
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .filter_map(|line| {
                        line.strip_prefix("HARDBORE_SELECTED:")
                            .map(|s| s.to_string())
                    })
                    .collect()
            }
            Ok(output) => {
                eprintln!(
                    "[HardBore Portal] Picker exited with code {:?}",
                    output.status.code()
                );
                eprintln!(
                    "[HardBore Portal] stderr: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                vec![]
            }
            Err(e) => {
                eprintln!("[HardBore Portal] Failed to launch picker: {}", e);
                vec![]
            }
        }
    }

    fn build_picker_args(
        mode: &str,
        multiple: bool,
        filters: &[FileFilter],
        current_folder: Option<&str>,
        current_name: Option<&str>,
    ) -> Vec<String> {
        let mut args = vec![mode.to_string()];

        if multiple {
            args.push("--multiple".to_string());
        }

        let extensions: Vec<String> = filters
            .iter()
            .flat_map(|f| {
                f.patterns.iter().filter_map(|p| {
                    p.strip_prefix("*.").map(|ext| ext.to_string())
                })
            })
            .collect();

        if !extensions.is_empty() {
            args.push("--types".to_string());
            args.push(extensions.join(","));
        }

        if let Some(folder) = current_folder {
            args.push("--start-dir".to_string());
            args.push(folder.to_string());
        }

        if let Some(name) = current_name {
            args.push("--current-name".to_string());
            args.push(name.to_string());
        }

        args
    }

    fn build_response(uris: Vec<String>) -> (u32, HashMap<String, OwnedValue>) {
        if uris.is_empty() {
            return (1, HashMap::new());
        }

        let mut result = HashMap::new();
        result.insert(
            "uris".to_string(),
            Value::new(uris).try_into().unwrap(),
        );
        (0, result)
    }
}

#[interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl FileChooserPortal {
    #[zbus(property, name = "version")]
    async fn version(&self) -> u32 {
        3
    }

    async fn open_file(
        &self,
        _handle: OwnedObjectPath,
        _app_id: &str,
        _parent_window: &str,
        _title: &str,
        options: HashMap<String, OwnedValue>,
    ) -> (u32, HashMap<String, OwnedValue>) {
        eprintln!("[HardBore Portal] OpenFile: app={} title={}", _app_id, _title);

        let multiple = extract_bool(&options, "multiple");
        let directory = extract_bool(&options, "directory");
        let current_folder = extract_current_folder(&options);
        let filters = parse_filters(&options);

        let mode = if directory { "--picker-dirs" } else { "--picker" };
        let args = Self::build_picker_args(
            mode,
            multiple,
            &filters,
            current_folder.as_deref(),
            None,
        );

        let selected = self.launch_picker(&args).await;
        let uris: Vec<String> = selected.iter().map(|p| encode_file_uri(p)).collect();
        Self::build_response(uris)
    }

    async fn save_file(
        &self,
        _handle: OwnedObjectPath,
        _app_id: &str,
        _parent_window: &str,
        _title: &str,
        options: HashMap<String, OwnedValue>,
    ) -> (u32, HashMap<String, OwnedValue>) {
        eprintln!("[HardBore Portal] SaveFile: app={} title={}", _app_id, _title);

        let current_folder = extract_current_folder(&options);
        let current_name = extract_current_name(&options);
        let filters = parse_filters(&options);

        let args = Self::build_picker_args(
            "--picker-save",
            false,
            &filters,
            current_folder.as_deref(),
            current_name.as_deref(),
        );

        let selected = self.launch_picker(&args).await;
        let uris: Vec<String> = selected.iter().map(|p| encode_file_uri(p)).collect();
        Self::build_response(uris)
    }

    async fn save_files(
        &self,
        _handle: OwnedObjectPath,
        _app_id: &str,
        _parent_window: &str,
        _title: &str,
        options: HashMap<String, OwnedValue>,
    ) -> (u32, HashMap<String, OwnedValue>) {
        eprintln!("[HardBore Portal] SaveFiles: app={} title={}", _app_id, _title);

        let current_folder = extract_current_folder(&options);
        let filenames = extract_filenames(&options);

        let args = Self::build_picker_args(
            "--picker-dirs",
            false,
            &[],
            current_folder.as_deref(),
            None,
        );

        let selected = self.launch_picker(&args).await;
        if selected.is_empty() {
            return (1, HashMap::new());
        }

        let chosen_dir = &selected[0];
        let uris: Vec<String> = if filenames.is_empty() {
            vec![encode_file_uri(chosen_dir)]
        } else {
            filenames
                .iter()
                .map(|name| encode_file_uri(&format!("{}/{}", chosen_dir, name)))
                .collect()
        };

        Self::build_response(uris)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[HardBore Portal] Starting XDG Desktop Portal backend...");

    let portal = FileChooserPortal::new();
    eprintln!("[HardBore Portal] HardBore binary: {}", portal.hardbore_path);

    let _connection = ConnectionBuilder::session()?
        .name("org.freedesktop.impl.portal.desktop.hardbore")?
        .serve_at(PORTAL_PATH, portal)?
        .build()
        .await?;

    eprintln!("[HardBore Portal] D-Bus service registered, waiting for requests...");

    std::future::pending::<()>().await;
    Ok(())
}
