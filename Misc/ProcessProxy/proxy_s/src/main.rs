use eframe::egui;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;

struct ServerProxyApp {
    listen_ip: String,
    listen_port: String,
    secret_code: String,
    status: Arc<Mutex<String>>,
    connections: Arc<Mutex<HashMap<String, u32>>>,
}

impl Default for ServerProxyApp {
    fn default() -> Self {
        Self {
            listen_ip: "127.0.0.1".to_string(),
            listen_port: "8080".to_string(),
            secret_code: "your_secret_code_here".to_string(),
            status: Arc::new(Mutex::new("Idle".to_string())),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl eframe::App for ServerProxyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Server Proxy Manager");
            
            ui.horizontal(|ui| {
                ui.label("Listen IP:");
                ui.text_edit_singleline(&mut self.listen_ip);
            });

            ui.horizontal(|ui| {
                ui.label("Listen Port:");
                ui.text_edit_singleline(&mut self.listen_port);
            });

            ui.horizontal(|ui| {
                ui.label("Secret Code:");
                ui.text_edit_singleline(&mut self.secret_code);
            });

            if ui.button("Start Server").clicked() {
                let listen_ip = self.listen_ip.clone();
                let listen_port = self.listen_port.clone();
                let secret_code = self.secret_code.clone();
                let status = self.status.clone();
                let connections = self.connections.clone();

                tokio::spawn(async move {
                    start_server(listen_ip, listen_port, secret_code, status, connections).await;
                });
            }

            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(self.status.lock().unwrap().clone());
            });

            ui.label("Active Connections:");
            for (addr, count) in self.connections.lock().unwrap().iter() {
                ui.label(format!("{}: {} connections", addr, count));
            }
        });

        ctx.request_repaint();
    }
}

async fn start_server(
    listen_ip: String,
    listen_port: String,
    secret_code: String,
    status: Arc<Mutex<String>>,
    connections: Arc<Mutex<HashMap<String, u32>>>,
) {
    let addr = format!("{}:{}", listen_ip, listen_port);
    *status.lock().unwrap() = format!("Starting server on {}", addr);

    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            *status.lock().unwrap() = format!("Error: {}", e);
            return;
        }
    };

    *status.lock().unwrap() = format!("Listening on {}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let secret_code = secret_code.clone();
                let connections = connections.clone();
                let status = status.clone();

                tokio::spawn(async move {
                    handle_connection(stream, addr, &secret_code, connections, status).await;
                });
            }
            Err(e) => {
                *status.lock().unwrap() = format!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    addr: std::net::SocketAddr,
    secret_code: &str,
    connections: Arc<Mutex<HashMap<String, u32>>>,
    status: Arc<Mutex<String>>,
) {
    let mut buffer = [0; 1024];
    
    // Read the secret code
    match stream.read(&mut buffer).await {
        Ok(n) => {
            if n == 0 || &buffer[..n] != secret_code.as_bytes() {
                let _ = stream.write_all(b"ERROR\r\n").await;
                *status.lock().unwrap() = format!("Invalid secret code from {}", addr);
                return;
            }
        }
        Err(_) => {
            *status.lock().unwrap() = format!("Error reading from {}", addr);
            return;
        }
    }

    // Send OK response
    if let Err(_) = stream.write_all(b"OK\r\n").await {
        *status.lock().unwrap() = format!("Error sending OK to {}", addr);
        return;
    }

    // Update connections count
    let addr_str = addr.to_string();
    connections.lock().unwrap()
        .entry(addr_str.clone())
        .and_modify(|e| *e += 1)
        .or_insert(1);

    *status.lock().unwrap() = format!("New connection from {}", addr);

    // Echo incoming data
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => break,
            Ok(n) => {
                if let Err(_) = stream.write_all(&buffer[..n]).await {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    // Decrease connections count
    let mut conns = connections.lock().unwrap();
    if let Some(count) = conns.get_mut(&addr_str) {
        *count -= 1;
        if *count == 0 {
            conns.remove(&addr_str);
        }
    }

    *status.lock().unwrap() = format!("Connection closed from {}", addr);
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let app = ServerProxyApp::default();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 300.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Server Proxy Manager",
        options,
        Box::new(|_cc| Box::new(app)),
    )
}