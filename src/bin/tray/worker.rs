//! Background worker: owns the `AppConfig` and a tokio runtime, fetches tasks
//! from all providers and performs close operations, so the UI thread never
//! blocks on network I/O.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

use eframe::egui;
use log::{debug, error};

use tskmstr::config::AppConfig;
use tskmstr::control::close_task;
use tskmstr::output::collect_all_tasks;
use tskmstr::providers::common::model::Issue;

pub enum Request {
    /// Re-fetch all tasks now
    Refresh,
    /// Close (complete) the task with this `<store_id>/<issue>` id
    Close(String),
}

pub enum Response {
    Tasks(Vec<Issue>),
    Closed { id: String, error: Option<String> },
    Error(String),
}

/// Spawn the worker thread. Tasks are refreshed immediately, then every
/// `refresh_interval`, and whenever a [`Request::Refresh`] arrives.
pub fn spawn(
    config: AppConfig,
    ctx: egui::Context,
    refresh_interval: Duration,
) -> (Sender<Request>, Receiver<Response>) {
    let (req_tx, req_rx) = channel::<Request>();
    let (resp_tx, resp_rx) = channel::<Response>();

    std::thread::Builder::new()
        .name("tskmstr-worker".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to start tokio runtime");

            let send = |r: Response| {
                let _ = resp_tx.send(r);
                ctx.request_repaint();
            };

            let refresh = || {
                debug!("refreshing tasks");
                match run_guarded(|| rt.block_on(collect_all_tasks(&None, &config))) {
                    Ok(issues) => send(Response::Tasks(issues)),
                    Err(e) => {
                        error!("refresh failed: {e}");
                        send(Response::Error(e));
                    }
                }
            };

            refresh();

            loop {
                match req_rx.recv_timeout(refresh_interval) {
                    Ok(Request::Refresh) | Err(RecvTimeoutError::Timeout) => refresh(),
                    Ok(Request::Close(id)) => {
                        let error =
                            run_guarded(|| rt.block_on(close_task(&config, id.clone()))).err();
                        if let Some(e) = &error {
                            error!("close {id} failed: {e}");
                        }
                        send(Response::Closed { id, error });
                        refresh();
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        })
        .expect("failed to spawn worker thread");

    (req_tx, resp_rx)
}

/// Run a provider call, converting both `Err` and panics (the provider code
/// panics on missing keyring credentials, for example) into a message.
fn run_guarded<T>(f: impl FnOnce() -> Result<T, anyhow::Error>) -> Result<T, String> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(e.to_string()),
        Err(panic) => {
            let msg = panic
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown error".to_string());
            Err(msg)
        }
    }
}
