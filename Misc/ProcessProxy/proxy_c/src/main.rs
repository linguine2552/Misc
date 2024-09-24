use eframe::egui;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;
use std::process::Command;
use std::sync::{Arc, Mutex};
use sysinfo::{System, SystemExt, ProcessExt};

struct ProxyApp {
    proxy_ip: String,
    proxy_port: String,
    secret_code: String,
    target_pid: String,
    status: Arc<Mutex<String>>,
    processes: Vec<(String, String)>,
    selected_process: Option<usize>,
}

impl Default for ProxyApp {
    fn default() -> Self {
        Self {
            proxy_ip: "127.0.0.1".to_string(),
            proxy_port: "8080".to_string(),
            secret_code: "your_secret_code_here".to_string(),
            target_pid: String::new(),
            status: Arc::new(Mutex::new("Idle".to_string())),
            processes: Vec::new(),
            selected_process: None,
        }
    }
}

impl ProxyApp {
    fn update_process_list(&mut self) {
        let mut sys = System::new_all();
        sys.refresh_all();

        self.processes = sys.processes()
            .iter()
            .map(|(pid, process)| (pid.to_string(), process.name().to_string()))
            .collect();
        self.processes.sort_by(|a, b| a.1.cmp(&b.1));
    }
}

impl eframe::App for ProxyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Network Proxy");
            
            ui.horizontal(|ui| {
                ui.label("Proxy IP:");
                ui.text_edit_singleline(&mut self.proxy_ip);
            });

            ui.horizontal(|ui| {
                ui.label("Proxy Port:");
                ui.text_edit_singleline(&mut self.proxy_port);
            });

            ui.horizontal(|ui| {
                ui.label("Secret Code:");
                ui.text_edit_singleline(&mut self.secret_code);
            });

            if ui.button("Refresh Process List").clicked() {
                self.update_process_list();
            }

            egui::ComboBox::from_label("Select Target Process")
                .selected_text(self.selected_process
                    .map(|i| self.processes[i].1.as_str())
                    .unwrap_or("Select a process"))
                .show_ui(ui, |ui| {
                    for (i, (pid, name)) in self.processes.iter().enumerate() {
                        ui.selectable_value(&mut self.selected_process, Some(i), format!("{} (PID: {})", name, pid));
                    }
                });

            if let Some(selected) = self.selected_process {
                self.target_pid = self.processes[selected].0.clone();
            }

            if ui.button("Start Proxy").clicked() {
                let proxy_ip = self.proxy_ip.clone();
                let proxy_port = self.proxy_port.clone();
                let secret_code = self.secret_code.clone();
                let target_pid = self.target_pid.clone();
                let status = self.status.clone();

                thread::spawn(move || {
                    start_proxy(proxy_ip, proxy_port, secret_code, target_pid, status);
                });
            }

            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(self.status.lock().unwrap().clone());
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let mut app = ProxyApp::default();
    app.update_process_list();

    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(400.0, 300.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Network Proxy",
        options,
        Box::new(|_cc| Box::new(app)),
    )
}

fn start_proxy(proxy_ip: String, proxy_port: String, secret_code: String, target_pid: String, status: Arc<Mutex<String>>) {
    *status.lock().unwrap() = "Starting...".to_string();

    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => {
            *status.lock().unwrap() = format!("Error: {}", e);
            return;
        }
    };
    let local_port = listener.local_addr().unwrap().port();
    
    *status.lock().unwrap() = format!("Listening on 127.0.0.1:{}", local_port);
    
    let target_pid = target_pid.parse::<u32>().unwrap();
    let target_ports = match get_pid_ports(target_pid) {
        Ok(ports) => ports,
        Err(e) => {
            *status.lock().unwrap() = format!("Error: {}", e);
            return;
        }
    };
    
    for &port in &target_ports {
        if let Err(e) = setup_port_forwarding(port, local_port) {
            *status.lock().unwrap() = format!("Error setting up port forwarding: {}", e);
            return;
        }
    }
    
    *status.lock().unwrap() = format!("Proxying for PID {} (ports: {:?})", target_pid, target_ports);

    // Connect to the proxy server
    let mut server_stream = match TcpStream::connect(format!("{}:{}", proxy_ip, proxy_port)) {
        Ok(stream) => stream,
        Err(e) => {
            *status.lock().unwrap() = format!("Error connecting to proxy server: {}", e);
            return;
        }
    };

    // Send the secret code
    if let Err(e) = server_stream.write_all(secret_code.as_bytes()) {
        *status.lock().unwrap() = format!("Error sending secret code: {}", e);
        return;
    }

    // Read the response
    let mut response = [0; 4];
    if let Err(e) = server_stream.read_exact(&mut response) {
        *status.lock().unwrap() = format!("Error reading server response: {}", e);
        return;
    }

    if &response != b"OK\r\n" {
        *status.lock().unwrap() = "Invalid server response".to_string();
        return;
    }

    *status.lock().unwrap() = "Connected to proxy server".to_string();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let proxy_ip = proxy_ip.clone();
        let proxy_port = proxy_port.clone();
        let secret_code = secret_code.clone();
        let status = status.clone();

        thread::spawn(move || {
            if let Err(e) = handle_connection(stream, &proxy_ip, &proxy_port, &secret_code) {
                *status.lock().unwrap() = format!("Connection error: {}", e);
            }
        });
    }
}

fn get_pid_ports(pid: u32) -> std::io::Result<Vec<u16>> {
    let output = Command::new("netstat")
        .args(&["-ano"])
        .output()?;
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut ports = Vec::new();
    
    for line in output_str.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 && parts[4] == pid.to_string() {
            if let Some(port_str) = parts[1].split(':').last() {
                if let Ok(port) = port_str.parse::<u16>() {
                    ports.push(port);
                }
            }
        }
    }
    
    Ok(ports)
}

fn setup_port_forwarding(from_port: u16, to_port: u16) -> std::io::Result<()> {
    Command::new("netsh")
        .args(&["interface", "portproxy", "add", "v4tov4", 
                &format!("listenport={}", from_port),
                "listenaddress=127.0.0.1",
                &format!("connectport={}", to_port),
                "connectaddress=127.0.0.1"])
        .output()?;
    
    Ok(())
}

fn handle_connection(mut client_stream: TcpStream, proxy_ip: &str, proxy_port: &str, secret_code: &str) -> std::io::Result<()> {
    let port = proxy_port.parse::<u16>().map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
    let mut server_stream = TcpStream::connect((proxy_ip, port))?;
    
    server_stream.write_all(secret_code.as_bytes())?;
    let mut response = [0; 4];
    server_stream.read_exact(&mut response)?;
    if &response != b"OK\r\n" {
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Handshake failed"));
    }
    
    let mut client_buffer = [0; 1024];
    let mut server_buffer = [0; 1024];

    loop {
        let client_bytes = match client_stream.read(&mut client_buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        server_stream.write_all(&client_buffer[..client_bytes])?;

        let server_bytes = match server_stream.read(&mut server_buffer) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => break,
        };
        client_stream.write_all(&server_buffer[..server_bytes])?;
    }
    
    Ok(())
}