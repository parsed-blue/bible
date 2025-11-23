use redis::{Client, Commands};
use rocket::{
    Request, Response,
    fairing::{Fairing, Info, Kind},
};
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;
use std::{env, time::Duration};

struct MemoryLog {
    control: Sender<Control>,
    data: Sender<String>,
}

#[derive(Debug)]
enum LogError {
    CouldNotOpenFile,
    CouldNotWriteToFile
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
    pub fn loop_logic(data_rx: Receiver<String>, control_rx: Receiver<Control>) -> Result<(), LogError> {
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

pub struct TrafficLog {
    client: Option<Client>,
    memory: MemoryLog,
}

impl Default for TrafficLog {
    fn default() -> Self {
        let client = env::var("REDIS_URL")
            .ok()
            .and_then(|redis_url| Client::open(redis_url).ok());

        if client.is_some() {
            println!("Redis client initialized successfully");
        }

        Self {
            client,
            memory: MemoryLog::new(),
        }
    }
}

impl TrafficLog {
    fn log(&self, user_agent: &str, user_ip: &str, path: &str, method: &str) {
        let log_data = json!({
            "user_agent": user_agent,
            "user_ip": user_ip,
            "path": path,
            "method": method,
        });
        if let Some(client) = self.client.as_ref() {
            let mut conn = client.get_connection().unwrap();
            let _: () = conn.publish("log", log_data.to_string()).unwrap();
        } else {
            self.memory.publish(log_data.to_string());
        }
    }
}

#[rocket::async_trait]
impl Fairing for TrafficLog {
    fn info(&self) -> Info {
        Info {
            name: "TrafficLog",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, _response: &mut Response<'r>) {
        let user_agent = request.headers().get_one("User-Agent").unwrap_or("");
        let user_ip = request
            .real_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let path = request.uri().path().as_str();
        let method = request.method().as_str();

        self.log(user_agent, &user_ip, path, method);
    }
}
