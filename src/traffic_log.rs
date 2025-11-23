use redis::{Client, Commands};
use rocket::{
    Request, Response,
    fairing::{Fairing, Info, Kind},
};
use serde_json::json;
use std::env;

pub struct TrafficLog {
    client: Option<Client>,
}

impl Default for TrafficLog {
    fn default() -> Self {
        let client = env::var("REDIS_URL")
            .ok()
            .and_then(|redis_url| Client::open(redis_url).ok());

        if client.is_some() {
            println!("Redis client initialized successfully");
        }

        Self { client }
    }
}

impl TrafficLog {
    fn log(&self, user_agent: &str, user_ip: &str, path: &str, method: &str) {
        if let Some(client) = self.client.as_ref() {
            let mut conn = client.get_connection().unwrap();

            let log_data = json!({
                "user_agent": user_agent,
                "user_ip": user_ip,
                "path": path,
                "method": method,
            });

            let _: () = conn.publish("log", log_data.to_string()).unwrap();
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
            .client_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let path = request.uri().path().as_str();
        let method = request.method().as_str();

        self.log(user_agent, &user_ip, path, method);
    }
}
