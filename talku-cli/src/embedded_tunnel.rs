//! In-process wstunnel client.
//!
//! Earlier builds shipped the pinned wstunnel release as a separate
//! `talku-wstunnel` executable and spawned it. wstunnel 10.5.1 is split into a
//! library (`wstunnel`) and a CLI (`wstunnel-cli`) that only parses arguments,
//! so the daemon links the library and feeds it the same command line the child
//! process used to receive. That keeps the pin in `Cargo.lock` instead of in a
//! downloaded binary, and removes the sibling-executable lookup entirely.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wstunnel::config::Client;
use wstunnel::executor::JoinSetTokioExecutor;

use crate::{log, TUNNEL_PORT};

/// A running wstunnel client. `is_finished` reports whether it stopped on its
/// own; `stop` (or dropping it) aborts the tunnel tasks.
pub struct Tunnel {
    // Shares its JoinSet with every task the client spawned, so aborting here
    // tears down the tunnel without waiting for a process to exit.
    executor: JoinSetTokioExecutor,
    finished: Arc<AtomicBool>,
    stopping: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Tunnel {
    /// Start the client against `wss://host:port`, forwarding local UDP
    /// `TUNNEL_PORT` to the same port on the far side — the same tunnel the
    /// daemon used to request from the standalone binary.
    pub fn start(host: &str, port: &str) -> Result<Self, String> {
        let client = build_client(host, port)?;
        let executor = JoinSetTokioExecutor::default();
        let finished = Arc::new(AtomicBool::new(false));
        let stopping = Arc::new(AtomicBool::new(false));

        let thread_executor = executor.clone();
        let thread_finished = finished.clone();
        let thread_stopping = stopping.clone();
        let thread = std::thread::Builder::new()
            .name("wstunnel".into())
            .spawn(move || {
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(e) => {
                        log(&format!("tunnel: failed to build runtime: {e}"));
                        thread_finished.store(true, Ordering::Relaxed);
                        return;
                    }
                };

                // `run_client` only returns once every tunnel task has ended,
                // either because the tunnel failed or because `abort_all` tore
                // the tasks down for an intentional `down`.
                let result = runtime.block_on(wstunnel::run_client(client, thread_executor));
                if !thread_stopping.load(Ordering::Relaxed) {
                    match result {
                        Ok(()) => log("tunnel: client stopped"),
                        Err(e) => log(&format!("tunnel: client failed: {e:#}")),
                    }
                }
                thread_finished.store(true, Ordering::Relaxed);
            })
            .map_err(|e| format!("failed to spawn tunnel thread: {e}"))?;

        Ok(Self {
            executor,
            finished,
            stopping,
            thread: Some(thread),
        })
    }

    /// True once the tunnel has stopped on its own (failed or exited).
    pub fn is_finished(&self) -> bool {
        self.finished.load(Ordering::Relaxed)
    }

    /// Abort the tunnel without blocking. The runtime shuts down on its own
    /// thread once the tasks are aborted, matching the old behaviour of killing
    /// the child process without waiting for it to exit.
    pub fn stop(mut self) {
        self.shutdown();
    }

    fn shutdown(&mut self) {
        self.stopping.store(true, Ordering::Relaxed);
        self.executor.abort_all();
        // std has no timed join; detaching keeps `down()` fast even if a task
        // is slow to reach its next await point.
        self.thread.take();
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Build the client config from the same command line the daemon used to pass
/// to `wstunnel client`, so every upstream default (TLS, retry/backoff, ping
/// interval, proxy environment variables) is exactly what the CLI would have
/// produced.
fn build_client(host: &str, port: &str) -> Result<Client, String> {
    use clap::{Args, FromArgMatches};

    let local = format!("udp://{TUNNEL_PORT}:localhost:{TUNNEL_PORT}?timeout_sec=0");
    let remote = format!("wss://{host}:{port}");
    let matches = Client::augment_args(clap::Command::new("wstunnel"))
        .try_get_matches_from(["wstunnel", "-L", &local, &remote])
        .map_err(|e| format!("wstunnel config: {e}"))?;
    Client::from_arg_matches(&matches).map_err(|e| format!("wstunnel config: {e}"))
}
