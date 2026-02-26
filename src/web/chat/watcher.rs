use actix::Addr;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::message::AssetChanged;
use super::server::ChatServer;

pub fn start_asset_watcher(chat_server: Addr<ChatServer>) {
    let asset_dir = std::env::var("CHAT_ASSET_DIR").unwrap_or_else(|_| ".".to_string());

    std::thread::spawn(move || {
        let (tx, rx) = mpsc::channel();

        let mut watcher =
            RecommendedWatcher::new(tx, Config::default()).expect("Failed to create file watcher");

        watcher
            .watch(Path::new(&asset_dir), RecursiveMode::NonRecursive)
            .expect("Failed to watch asset directory");

        let mut last_notify = Instant::now();

        log::info!("Asset watcher started on: {}", asset_dir);

        for res in rx {
            match res {
                Ok(event) => {
                    // Only care about create/modify events on .js/.css files
                    let dominated =
                        matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_));
                    let relevant_file = event.paths.iter().any(|p| {
                        p.extension()
                            .map(|ext| ext == "js" || ext == "css")
                            .unwrap_or(false)
                    });

                    if dominated && relevant_file {
                        // Debounce: don't send more than once per 2 seconds
                        if last_notify.elapsed() > Duration::from_secs(2) {
                            last_notify = Instant::now();
                            log::info!("Asset change detected: {:?}", event.paths);
                            chat_server.do_send(AssetChanged);
                        }
                    }
                }
                Err(e) => log::warn!("File watcher error: {:?}", e),
            }
        }
    });
}
