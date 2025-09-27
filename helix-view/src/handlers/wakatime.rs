use crate::document::Document;
use crate::events::DocumentDidOpen;
use crate::handlers::Handlers;
use crate::ViewId;
use helix_event::register_hook;
use parking_lot::RwLock;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// WakaTime event types
#[derive(Debug, Clone)]
pub enum WakaTimeEvent {
    /// A heartbeat event for WakaTime tracking
    Heartbeat {
        entity: String,
        type_: WakaTimeEntityType,
        category: WakaTimeCategory,
        time: f64,
        project: Option<String>,
        language: Option<String>,
        is_write: bool,
        lines: Option<u32>,
        lineno: Option<u32>,
        cursorpos: Option<u32>,
    },
}

#[derive(Debug, Clone)]
pub enum WakaTimeEntityType {
    File,
    Domain,
    App,
}

#[derive(Debug, Clone)]
pub enum WakaTimeCategory {
    Coding,
    Building,
    Indexing,
    Debugging,
    Running,
    Testing,
    Manual,
    Writing,
    Designing,
    Researching,
}

impl std::fmt::Display for WakaTimeEntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WakaTimeEntityType::File => write!(f, "file"),
            WakaTimeEntityType::Domain => write!(f, "domain"),
            WakaTimeEntityType::App => write!(f, "app"),
        }
    }
}

impl std::fmt::Display for WakaTimeCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WakaTimeCategory::Coding => write!(f, "coding"),
            WakaTimeCategory::Building => write!(f, "building"),
            WakaTimeCategory::Indexing => write!(f, "indexing"),
            WakaTimeCategory::Debugging => write!(f, "debugging"),
            WakaTimeCategory::Running => write!(f, "running"),
            WakaTimeCategory::Testing => write!(f, "testing"),
            WakaTimeCategory::Manual => write!(f, "manual"),
            WakaTimeCategory::Writing => write!(f, "writing"),
            WakaTimeCategory::Designing => write!(f, "designing"),
            WakaTimeCategory::Researching => write!(f, "researching"),
        }
    }
}

/// WakaTime handler for tracking coding activity
pub struct Handler {
    pub sender: UnboundedSender<WakaTimeEvent>,
    config: Arc<RwLock<Option<crate::editor::WakaTimeConfig>>>,
}

/// WakaTime worker that processes events and sends them to the API
struct Worker {
    receiver: UnboundedReceiver<WakaTimeEvent>,
    #[cfg(feature = "wakatime")]
    client: Option<reqwest::Client>,
    config: Arc<RwLock<Option<crate::editor::WakaTimeConfig>>>,
}

impl Handler {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded_channel();
        let config = Arc::new(RwLock::new(None));
        let config_clone = config.clone();
        
        // Spawn the worker task
        tokio::spawn(async move {
            let mut worker = Worker {
                receiver,
                #[cfg(feature = "wakatime")]
                client: None,
                config: config_clone,
            };
            worker.run().await;
        });

        Self { sender, config }
    }

    /// Update the WakaTime configuration
    pub fn update_config(&self, wakatime_config: crate::editor::WakaTimeConfig) {
        *self.config.write() = Some(wakatime_config);
    }

    /// Send a heartbeat event to WakaTime
    pub fn send_heartbeat(&self, event: WakaTimeEvent) {
        if let Err(e) = self.sender.send(event) {
            log::warn!("Failed to send WakaTime event: {}", e);
        }
    }
}

impl Worker {
    async fn run(&mut self) {
        #[cfg(feature = "wakatime")]
        {
            self.client = Some(reqwest::Client::new());
        }

        while let Some(event) = self.receiver.recv().await {
            self.process_event(event).await;
        }
    }

    async fn process_event(&self, event: WakaTimeEvent) {
        let config_guard = self.config.read();
        let Some(config) = config_guard.as_ref() else {
            log::debug!("WakaTime config not available, skipping event");
            return;
        };

        if !config.enabled {
            return;
        }

        let Some(api_key) = &config.api_key else {
            log::warn!("WakaTime API key not configured");
            return;
        };

        #[cfg(feature = "wakatime")]
        {
            if let Some(client) = &self.client {
                match event {
                    WakaTimeEvent::Heartbeat {
                        entity,
                        type_,
                        category,
                        time,
                        project,
                        language,
                        is_write,
                        lines,
                        lineno,
                        cursorpos,
                    } => {
                        let payload = json!({
                            "entity": entity,
                            "type": type_.to_string(),
                            "category": category.to_string(),
                            "time": time,
                            "project": project,
                            "language": language,
                            "is_write": is_write,
                            "lines": lines,
                            "lineno": lineno,
                            "cursorpos": cursorpos,
                        });

                        let auth_header = format!("Bearer {}", api_key);
                        let timeout = std::time::Duration::from_secs(config.timeout);

                        match client
                            .post(&config.api_url)
                            .header("Authorization", &auth_header)
                            .header("Content-Type", "application/json")
                            .header("User-Agent", "helix-editor")
                            .timeout(timeout)
                            .json(&payload)
                            .send()
                            .await
                        {
                            Ok(response) => {
                                if response.status().is_success() {
                                    log::debug!("WakaTime heartbeat sent successfully");
                                } else {
                                    log::warn!(
                                        "WakaTime API returned status: {}",
                                        response.status()
                                    );
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to send WakaTime heartbeat: {}", e);
                            }
                        }
                    }
                }
            }
        }
        
        #[cfg(not(feature = "wakatime"))]
        {
            log::debug!("WakaTime feature not enabled, would send: {:?}", event);
        }
    }
}

/// Get the current Unix timestamp as a float
fn current_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

/// Extract language name from a document
fn get_language_name(doc: &Document) -> Option<String> {
    doc.language_config()
        .map(|config| config.language_id.clone())
}

/// Extract project name from a file path
fn get_project_name(path: &PathBuf) -> Option<String> {
    // Simple heuristic: look for common project root indicators
    let mut current = path.as_path();
    while let Some(parent) = current.parent() {
        for indicator in &[".git", ".hg", ".svn", "Cargo.toml", "package.json", "pyproject.toml"] {
            if parent.join(indicator).exists() {
                return parent
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|s| s.to_string());
            }
        }
        current = parent;
    }
    None
}

/// Send a heartbeat for document activity
fn send_document_heartbeat(
    sender: &UnboundedSender<WakaTimeEvent>,
    doc: &Document,
    view_id: crate::ViewId,
    is_write: bool,
    wakatime_config: &crate::editor::WakaTimeConfig,
) {
    if !wakatime_config.enabled {
        return;
    }

    let Some(path) = doc.path() else {
        return;
    };

    let entity = path.to_string_lossy().to_string();
    let language = get_language_name(doc);
    let project = wakatime_config
        .project
        .clone()
        .or_else(|| get_project_name(path));

    let lines = doc.text().len_lines() as u32;
    // For now, we'll use the primary selection from the document's selections
    let selection = doc.selection(view_id);
    let cursor_pos = selection.primary().cursor(doc.text().slice(..));
    let line_idx = doc.text().char_to_line(cursor_pos);

    let event = WakaTimeEvent::Heartbeat {
        entity: if wakatime_config.hide_file_names {
            "HIDDEN".to_string()
        } else {
            entity
        },
        type_: WakaTimeEntityType::File,
        category: WakaTimeCategory::Coding,
        time: current_timestamp(),
        project: if wakatime_config.hide_project_names {
            None
        } else {
            project
        },
        language,
        is_write,
        lines: Some(lines),
        lineno: Some(line_idx as u32 + 1), // 1-indexed
        cursorpos: Some(cursor_pos as u32),
    };

    if let Err(e) = sender.send(event) {
        log::warn!("Failed to send WakaTime heartbeat: {}", e);
    }
}

/// Register WakaTime event hooks
pub(crate) fn register_hooks(handlers: &Handlers) {
    let Some(wakatime_handler) = handlers.wakatime.as_ref() else {
        return;
    };

    let sender = wakatime_handler.sender.clone();
    
    // Hook for document opens - this is the simplest case as it has all the context we need
    register_hook!(move |event: &mut DocumentDidOpen<'_>| {
        let doc = event.editor.documents.get(&event.doc).unwrap();
        // Use the focused view or get any view from the document
        let view_id = event.editor.tree.focus;
        send_document_heartbeat(&sender, doc, view_id, false, &event.editor.config().wakatime);
        Ok(())
    });
}