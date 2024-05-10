use std::net::SocketAddr;

use axum::Router;
use tokio::{
    net::TcpListener,
    spawn,
    sync::oneshot::{channel, Sender},
    task::JoinHandle,
};

use crate::fakehub::{Error, Result};

/// A temporarily running Axum server which stops when it is dropped, or
/// when the associated runtime shuts down. Whichever happens first.
#[derive(Debug)]
pub struct TempServer {
    pub socket_addr: SocketAddr,
    shutdown_channel: Option<Sender<()>>,
    completion_handle: JoinHandle<()>,
}

impl TempServer {
    // Start the server.
    pub async fn new(starting_port: u16, app: Router) -> Result<Self> {
        let port = port_selector::select_from_given_port(starting_port)
            .ok_or(Error::NoAvailablePorts(starting_port))?;
        let socket = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(socket)
            .await
            .map_err(|e| Error::Bind(e.to_string()))?;
        let server = axum::serve(listener, app.into_make_service());
        let (tx, rx) = channel::<()>();
        let graceful = server.with_graceful_shutdown(async {
            rx.await.ok();
        });

        let handle = spawn(async {
            if let Err(e) = graceful.await {
                eprintln!("server error: {}", e);
            }
        });

        Ok(Self {
            socket_addr: socket,
            shutdown_channel: Some(tx),
            completion_handle: handle,
        })
    }

    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_channel.take() {
            let _ = tx.send(());
        }

        self.completion_handle.await.ok();
    }
}
