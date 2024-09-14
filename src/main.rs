#![allow(unused_variables, dead_code)]

use std::fs;
use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::time::Duration;
use std::thread;
use std::collections::HashMap;
mod threads;
mod verify;
mod database;
use crate::threads::ThreadPool;
use sha256::digest;


fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    let pool = ThreadPool::new(10);
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        pool.execute(|| {
            Router::connection(stream)
        });
    }
    
}

#[derive(Debug)]
enum Method {
    GET,
    POST,
    OPTIONS,
    HEAD,
    PUT,
    DELETE,
}

#[derive(Debug)]
struct HTTP {
    method: Method,
    path: String,
    params: HashMap<String, String>,
    headers: HashMap<String, String>,
    data: String,
}

impl HTTP {
    fn new(request: &str) -> Option<Self> {
        let mut lines = request.lines();
        let first_line = lines.next()?.split_whitespace().collect::<Vec<&str>>();

        if first_line.len() != 3 {
            return None;
        }

        let method = match first_line[0] {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "OPTIONS" => Method::OPTIONS,
            "HEAD" => Method::HEAD,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            _ => return None,
        };

        let (path, params) = {
            let mut split = first_line[1].splitn(2, '?');
            let path = split.next().unwrap_or("").to_string();
            let query = split.next().unwrap_or("");
            let params = query.split('&')
                .filter_map(|param| {
                    let mut kv = param.splitn(2, '=');
                    Some((kv.next()?.to_string(), kv.next()?.to_string()))
                })
                .collect();
            (path, params)
        };

        let mut headers = HashMap::new();
        let mut data = String::new();
        let mut in_headers = true;

        for line in lines {
            if line.is_empty() {
                in_headers = false;
                continue;
            }

            if in_headers {
                if let Some((key, value)) = line.split_once(':') {
                    headers.insert(key.trim().to_string(), value.trim().to_string());
                }
            } else {
                data.push_str(line);
            }
        }

        Some(Self {
            method,
            path,
            params,
            headers,
            data: data.trim_matches('\0').to_string(),
        })
    }
}


struct Router;

impl Router {
    
    fn file_contents(filename: &str) -> Vec<u8> {
        fs::read(filename).unwrap()
    }

    fn respond(status: &str, data: Vec<u8>, content_type: &str) -> Vec<u8> {
        let header = format!(
            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            status,
            content_type,
            data.len()
        );
        let mut response = Vec::new();
        response.extend(header.as_bytes());
        response.extend(data);
        response
    }

    fn route(http: HTTP) -> Vec<u8> {

        let (status, data, content_type) = match (http.path.as_str(), http.method) {

            ("/", Method::GET) => {
                let data = Self::file_contents("index.html");
                ("200 OK", data, "text/html")
            }
    
            ("/sleep", Method::GET) => {
                thread::sleep(Duration::from_secs(5));
                let data = Self::file_contents("index.html");
                ("200 OK", data, "text/html")
            }

            ("/sign", Method::POST) => {

                let parts: Vec<&str> = http.data.split('|').collect();
                if parts.len() != 2 {
                    return Self::respond("400 Bad Request", "Invalid data format".into(), "text/plain");
                }

                let message = parts[0];
                let priv_key = parts[1];

                let hash = digest(message);
                
                match verify::sign(priv_key, &hash) {

                    Ok(sign) => ("200 OK", format!("{:x}", sign).into_bytes(), "text/plain"),

                    Err(e) => ("500 Internal Server Error", e.into_bytes(), "text/plain")

                }
            }

            ("/register", Method::GET) => {

                let (pub_key, priv_key) = verify::create_key();
                let data = format!("{:x}:{:x}", pub_key.to_bytes(), priv_key.to_bytes());
                
                ("200 OK", data.into_bytes(), "text/plain")
            }
            
            ("/register", Method::POST) => {

                let parts: Vec<&str> = http.data.split(':').collect();
                if parts.len() != 2 {
                    return Self::respond("400 Bad Request", "Invalid data format".into(), "text/plain");
                }
    
                let pub_key = parts[0];
                let sign = parts[1];
                let hash = digest(pub_key);


                match verify::verify(pub_key, &hash, sign) {

                    Ok(valid) => {
                        
                        if valid {
                            match database::register(pub_key) {
                                Ok(()) => ("200 OK", "User registered successfully".into(), "text/plain"),
                                Err(e) => ("500 Internal Server Error", e.into(), "text/plain"),
                            }
                        } else {
                            ("401 Unauthorized", "Invalid signature".into(), "text/plain")
                        }
                    }
                    Err(e) => ("500 Internal Server Error", e.into(), "text/plain"),
                }
            }

            ("/post", Method::GET) => {

                if let (Some(subject),Some(time)) = (http.params.get("sub"), http.params.get("t")){

                    let default_post_num = 10;
                    let post_num = http.params.get("n")
                    .map(|n| n.parse::<u8>().unwrap_or(default_post_num))
                    .unwrap_or(default_post_num);

                    if let Some(posts) = database::get_posts(subject, time, post_num){

                        ("200 OK", posts.into(), "application/json")
                    }
                    else {

                        ("404 Not Found", "None".into(), "text/plain")
                    }
                    
                } else {
                    
                    ("422 Unprocessable Content", "Missing paramters".into() , "text/plain")
                }

            }
    
            ("/post", Method::POST) => {

                let parts: Vec<&str> = http.data.split(':').collect();
                if parts.len() != 5 {
                    return Self::respond("400 Bad Request", "Invalid data format".into(), "text/plain");
                }
    
                let pub_key = parts[0];
                let subject = parts[1];
                let message = parts[2];
                let time = parts[3];
                let sign = parts[4];
    
                let hash_str = format!("{pub_key}:{subject}:{message}:{time}");
                let hash = digest(hash_str);
                let valid = verify::verify(pub_key, &hash, sign);

                match verify::verify(pub_key, &hash, sign) {

                    Ok(valid) => {
                        if valid {

                            match database::post(pub_key, subject, message, time, sign, &hash) {
                                Ok(()) => ("200 OK", "Posted successfully".into(), "text/plain"),
                                Err(e) => ("500 Internal Server Error", e.into(), "text/plain"),
                            }
                        } 
                        else {
                            ("401 Unauthorized", "Invalid signature".into(), "text/plain")
                        }
                    }
                    Err(e) => ("500 Internal Server Error", e.into(), "text/plain"),
                }

            }

            ("/time", Method::GET) => {

                ("200 OK", database::get_time().into(), "text/plain")

            }
    
            _ => {

                let path = format!(".{}", http.path);
                
                if path.contains("../") {
                    Self::respond("404 Not Found", "???".into(), "text/plain");
                }

                let extension = path.split('.').last().unwrap_or("");

                let content_type = match extension {
                    "html" => "text/html",
                    "css" => "text/css",
                    "js" => "application/javascript",
                    "png" => "image/png",
                    "jpg" | "jpeg" => "image/jpeg",
                    "gif" => "image/gif",
                    "ico" => "image/x-icon",
                    "svg" => "image/svg+xml",
                    "json" => "application/json",
                    _ => "application/octet-stream",
                };
            
                if let Ok(data) = fs::read(&path) {
                    ("200 OK", data, content_type)
                } else {
                    let data = Self::file_contents("404.html");
                    ("404 Not Found", data, "text/html")
                }
            }
            
        };
    
        Self::respond(status, data, content_type)
    }

    fn connection(mut stream: TcpStream) {

        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
    
        let request = String::from_utf8_lossy(&buffer[..]); 
    
        let response = if let Some(http) = HTTP::new(&request) {
            Self::route(http)
        } else {
            Self::respond("400 Bad Request", "".into(), "text/plain")
        };
        
        stream.write(response.as_slice()).unwrap();
        stream.flush().unwrap();
    }

}
