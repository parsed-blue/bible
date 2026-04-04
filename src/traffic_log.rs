use axum::{
    extract::{ConnectInfo, Request},
    middleware::Next,
    response::IntoResponse,
};
use redis::{Client, Commands};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::{env, time::Duration};

struct MemoryLog {
    control: Sender<Control>,
    data: Sender<String>,
}

#[derive(Debug)]
enum LogError {
    CouldNotOpenFile,
    CouldNotWriteToFile,
}

enum Control {
    Halt,
}

impl MemoryLog {
    pub fn close(&self) {
        if let Err(err) = self.control.send(Control::Halt) {
            eprintln!("could not publish control message {:?}", err);
        }
    }
    pub fn publish(&self, data: String) {
        if let Err(err) = self.data.send(data) {
            eprintln!("could not publish message {:?}", err);
        }
    }
    pub fn loop_logic(
        data_rx: Receiver<String>,
        control_rx: Receiver<Control>,
    ) -> Result<(), LogError> {
        let Ok(mut file) = OpenOptions::new()
            .append(true)
            .create(true)
            .open("access.jsonl")
        else {
            return Err(LogError::CouldNotOpenFile);
        };

        loop {
            if let Ok(control) = control_rx.try_recv() {
                match control {
                    Control::Halt => {
                        return Ok(());
                    }
                }
            }

            if let Ok(data) = data_rx.try_recv() {
                let Ok(()) = writeln!(file, "{}", data) else {
                    return Err(LogError::CouldNotWriteToFile);
                };
            }

            thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn new() -> MemoryLog {
        let (control_tx, control_rx) = mpsc::channel::<Control>();
        let (data_tx, data_rx) = mpsc::channel::<String>();

        thread::spawn(move || match MemoryLog::loop_logic(data_rx, control_rx) {
            Ok(_) => println!("loop closed successfully"),
            Err(e) => eprintln!("Loop closed with an issue: {:?}", e),
        });

        MemoryLog {
            control: control_tx,
            data: data_tx,
        }
    }
}

pub struct TrafficLogInner {
    client: Option<Client>,
    memory: MemoryLog,
}

#[derive(Clone)]
pub struct TrafficLog(Arc<TrafficLogInner>);

impl Default for TrafficLog {
    fn default() -> Self {
        let client = env::var("REDIS_URL")
            .ok()
            .and_then(|redis_url| Client::open(redis_url).ok());

        if client.is_some() {
            println!("Redis client initialized successfully");
        }

        Self(Arc::new(TrafficLogInner {
            client,
            memory: MemoryLog::new(),
        }))
    }
}

impl TrafficLog {
    pub fn log(&self, user_agent: &str, user_ip: &str, path: &str, method: &str) {
        let log_data = json!({
            "user_agent": user_agent,
            "user_ip": user_ip,
            "path": path,
            "method": method,
        });
        if let Some(client) = self.0.client.as_ref() {
            if let Ok(mut conn) = client.get_connection() {
                let _: Result<(), _> = conn.publish("log", log_data.to_string());
            } else {
                self.0.memory.publish(log_data.to_string());
            }
        } else {
            self.0.memory.publish(log_data.to_string());
        }
    }
}

pub async fn track_traffic(req: Request, next: Next) -> impl IntoResponse {
    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let user_ip = req
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            req.extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ConnectInfo(addr)| addr.ip().to_string())
                .unwrap_or_else(|| "unknown".to_string())
        });

    let path = req.uri().path().to_string();
    let method = req.method().as_str().to_string();

    let traffic_log = req.extensions().get::<TrafficLog>().cloned();

    let response = next.run(req).await;

    if let Some(log) = traffic_log {
        log.log(&user_agent, &user_ip, &path, &method);
    }

    response
}
