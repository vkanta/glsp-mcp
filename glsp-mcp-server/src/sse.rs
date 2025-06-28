use axum::{
    response::sse::{Event, KeepAlive, Sse},
    extract::State,
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use crate::wasm::FileSystemWatcher;

pub async fn wasm_changes_stream(
    State(watcher): State<Arc<RwLock<FileSystemWatcher>>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("New SSE connection for WASM changes");
    
    let watcher = watcher.read().await;
    let changes_rx = watcher.get_changes_receiver();
    
    let stream = async_stream::stream! {
        // Send initial connected event
        yield Ok(Event::default()
            .event("connected")
            .data("Connected to WASM file watcher"));
        
        // Get the receiver lock and consume messages
        let mut rx = changes_rx.write().await;
        
        while let Some(change) = rx.recv().await {
            match serde_json::to_string(&change) {
                Ok(json) => {
                    info!("Sending WASM change event: {:?}", change.event_type);
                    yield Ok(Event::default()
                        .event("wasm-change")
                        .data(json));
                }
                Err(e) => {
                    error!("Failed to serialize change event: {}", e);
                }
            }
        }
        
        info!("SSE connection closed");
    };
    
    Sse::new(stream).keep_alive(KeepAlive::default())
}