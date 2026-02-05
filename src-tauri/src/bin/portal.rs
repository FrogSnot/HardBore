use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
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

impl FileChooserPortal {
    pub fn new() -> Self {
        let hardbore_path = if std::path::Path::new("/usr/local/bin/hardbore").exists() {
            "/usr/local/bin/hardbore".to_string()
        } else if std::path::Path::new("/usr/bin/hardbore").exists() {
            "/usr/bin/hardbore".to_string()
        } else if let Ok(output) = Command::new("which").arg("hardbore").output() {
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

    fn launch_picker(
        &self,
        multiple: bool,
        directory: bool,
        filters: Vec<FileFilter>,
    ) -> Vec<String> {
        let mut args = vec![];
        if directory {
            args.push("--picker-dirs".to_string());
        } else {
            args.push("--picker".to_string());
        }

        if multiple {
            args.push("--multiple".to_string());
        }

        if !filters.is_empty() && !directory {
            let extensions: Vec<String> = filters
                .iter()
                .flat_map(|f| {
                    f.patterns
                        .iter()
                        .filter_map(|p| {
                            if p.starts_with("*.") {
                                Some(p.trim_start_matches("*.").to_string())
                            } else {
                                None
                            }
                        })
                })
                .collect();

            if !extensions.is_empty() {
                args.push("--types".to_string());
                args.push(extensions.join(","));
            }
        }

        eprintln!("[HardBore Portal] Launching: {} {:?}", &self.hardbore_path, &args);
        
        let output = Command::new(&self.hardbore_path)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match output {
            Ok(output) if output.status.success() => {
                eprintln!("[HardBore Portal] Command succeeded");
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .filter_map(|line| {
                        if line.starts_with("HARDBORE_SELECTED:") {
                            Some(line.trim_start_matches("HARDBORE_SELECTED:").to_string())
                        } else {
                            None
                        }
                    })
                    .collect()
            }
            Ok(output) => {
                eprintln!("[HardBore Portal] Command failed with exit code: {:?}", output.status.code());
                eprintln!("[HardBore Portal] Stdout: {}", String::from_utf8_lossy(&output.stdout));
                eprintln!("[HardBore Portal] Stderr: {}", String::from_utf8_lossy(&output.stderr));
                vec![]
            }
            Err(e) => {
                eprintln!("[HardBore Portal] Failed to execute command: {}", e);
                eprintln!("[HardBore Portal] Tried to run: {} {:?}", &self.hardbore_path, &args);
                vec![]
            }
        }
    }
}

#[interface(name = "org.freedesktop.impl.portal.FileChooser")]
impl FileChooserPortal {
    async fn open_file(
        &self,
        _handle: OwnedObjectPath,
        _app_id: &str,
        _parent_window: &str,
        _title: &str,
        options: HashMap<String, OwnedValue>,
    ) -> (u32, HashMap<String, OwnedValue>) {
        eprintln!("[HardBore Portal] OpenFile called");
        eprintln!("  App ID: {}", _app_id);
        eprintln!("  Title: {}", _title);
        eprintln!("  Options: {:?}", options);

        let multiple = match options.get("multiple") {
            Some(v) => match v.downcast_ref::<bool>() {
                Ok(b) => b,
                Err(_) => false,
            },
            None => false,
        };

        let directory = match options.get("directory") {
            Some(v) => match v.downcast_ref::<bool>() {
                Ok(b) => b,
                Err(_) => false,
            },
            None => false,
        };

        let filters: Vec<FileFilter> = options
            .get("filters")
            .and_then(|_v| {

                Some(vec![])
            })
            .unwrap_or_default();

        let selected = self.launch_picker(multiple, directory, filters);

        if selected.is_empty() {
            (1, HashMap::new())
        } else {
            let mut result = HashMap::new();
            let uris: Vec<String> = selected
                .iter()
                .map(|path| format!("file://{}", path))
                .collect();

            result.insert(
                "uris".to_string(),
                Value::new(uris).try_into().unwrap(),
            );

            (0, result)
        }
    }

    async fn save_file(
        &self,
        _handle: OwnedObjectPath,
        _app_id: &str,
        _parent_window: &str,
        _title: &str,
        _options: HashMap<String, OwnedValue>,
    ) -> (u32, HashMap<String, OwnedValue>) {
        eprintln!("[HardBore Portal] SaveFile called");
        
        let selected = self.launch_picker(false, false, vec![]);

        if selected.is_empty() {
            (1, HashMap::new())
        } else {
            let mut result = HashMap::new();
            let uris: Vec<String> = selected
                .iter()
                .map(|path| format!("file://{}", path))
                .collect();

            result.insert(
                "uris".to_string(),
                Value::new(uris).try_into().unwrap(),
            );

            (0, result)
        }
    }

    async fn save_files(
        &self,
        _handle: OwnedObjectPath,
        _app_id: &str,
        _parent_window: &str,
        _title: &str,
        _options: HashMap<String, OwnedValue>,
    ) -> (u32, HashMap<String, OwnedValue>) {
        eprintln!("[HardBore Portal] SaveFiles called");
        
        let selected = self.launch_picker(true, false, vec![]);

        if selected.is_empty() {
            (1, HashMap::new())
        } else {
            let mut result = HashMap::new();
            let uris: Vec<String> = selected
                .iter()
                .map(|path| format!("file://{}", path))
                .collect();

            result.insert(
                "uris".to_string(),
                Value::new(uris).try_into().unwrap(),
            );

            (0, result)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("[HardBore Portal] Starting XDG Desktop Portal backend...");

    let portal = FileChooserPortal::new();
    eprintln!("[HardBore Portal] HardBore path: {}", portal.hardbore_path);

    let _connection = ConnectionBuilder::session()?
        .name("org.freedesktop.impl.portal.desktop.hardbore")?
        .serve_at(PORTAL_PATH, portal)?
        .build()
        .await?;

    eprintln!("[HardBore Portal] D-Bus service registered");
    eprintln!("[HardBore Portal] Waiting for file chooser requests...");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
