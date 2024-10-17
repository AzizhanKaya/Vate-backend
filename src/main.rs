#![allow(unused_variables, dead_code)]

#[cfg(test)]
mod tests;

use std::fs;
use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::time::Duration;
use std::thread;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
mod threads;
mod verify;
mod database;
use crate::threads::ThreadPool;
use sha256::digest;
use regex::Regex;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
    let pool = ThreadPool::new(5);
    
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
    data: Vec<u8>,
}

impl HTTP {
    fn new(request: &[u8]) -> Option<Self> {

        let request_str = String::from_utf8_lossy(request);
        let mut lines = request_str.lines();
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
        
        
        for line in lines {

            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                headers.insert(key.trim().to_string(), value.trim().to_string());
            }
        }

        let mut data = Vec::new();
        let data_start_index = request.windows(4).position(|window| window == b"\r\n\r\n")? + 4;
        data.extend_from_slice(&request[data_start_index..]);

        
        Some(Self {
            method,
            path,
            params,
            headers,
            data: data
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Post {
    past_hash: Option<String>,
    pub_key: String,
    subject: String,
    message: String,
    time: String,
    sign: String,
    post: Option<Box<Post>>,
}

impl Post {

    pub fn new(data: &str) -> Option<Self> {
        
        let posts: Post = serde_json::from_str(data).ok()?;

        Some(posts)
    }

    pub fn last(&self) -> &Post {
        let mut current = self;
        while let Some(ref next_post) = current.post {
            current = next_post;
        }
        current
    }

    pub fn lenght(&self) -> u16 {
        let mut len = 1;
        let mut current = self;
        while let Some(ref next_post) = current.post {
            len += 1;
            current = next_post;
        }
        len
    }

    pub fn hash(&mut self) -> String {

        let hash: String = digest(format!("{}:{}:{}:{}:{}", 
            self.past_hash.as_ref().unwrap(),
            self.pub_key,
            self.subject,
            self.message,
            self.time
        ));

        if let Some(ref mut post) = self.post {
            post.past_hash = Some(hash);
            Self::hash(post)
        } else {
            hash
        }

    }

    pub fn iter(&self) -> PostIterator {
        PostIterator {
            current: Some(self),
        }
    }

    pub fn format(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.pub_key,
            self.subject,
            self.message,
            self.time,
            self.sign
        )
    }

}

pub struct PostIterator<'a> {
    current: Option<&'a Post>,
}

impl<'a> Iterator for PostIterator<'a> {
    type Item = &'a Post;

    fn next(&mut self) -> Option<Self::Item> {
        let current_post = self.current?;
        self.current = current_post.post.as_deref();
        Some(current_post)
    }
}


struct Router;

macro_rules! hmap {
    ( $( $key:expr => $value:expr ),* ) => {{
        let mut map = HashMap::new();
        $( map.insert($key, $value); )*
        map
    }};
}

impl Router {
    
    fn file_contents(filename: &str) -> Vec<u8> {
        fs::read(filename).unwrap()
    }

    fn respond(status: &str, data: Vec<u8>, headers: HashMap<&str, &str>) -> Vec<u8> {

        let mut response_header = format!(
            "HTTP/1.1 {}\r\nContent-Length: {}\r\n",
            status,
            data.len()
        );

        for (key, value) in headers {
            response_header.push_str(&format!("{}: {}\r\n", key, value));
        }

        response_header.push_str("\r\n");

        let mut response = Vec::new();
        response.extend(response_header.as_bytes());
        response.extend(data);
        response
    }

    fn route(http: HTTP) -> Vec<u8> {

        let (status, data) = match (http.path.as_str(), http.method) {

            ("/", Method::GET) => {
                let data = Self::file_contents("index.html");
                ("200 OK", data)
            }
    
            ("/sleep", Method::GET) => {
                thread::sleep(Duration::from_secs(5));
                let data = Self::file_contents("index.html");
                ("200 OK", data)
            }

            ("/sign", Method::POST) => {

                let data_string = String::from_utf8(http.data.to_vec()).unwrap_or_default();
                let parts: Vec<&str> = data_string.split('|').collect();

                if parts.len() != 2 {
                    return Self::respond("400 Bad Request", "Invalid data format".into(), hmap!{"Content-Type"=>"text/plain"})
                }

                let message = parts[0];
                let priv_key = parts[1];

                let hash = digest(message);
                
                match verify::sign(priv_key.into(), &hash) {

                    Ok(sign) => ("200 OK", format!("{:x}", sign).into_bytes() ),

                    Err(e) => ("500 Internal Server Error", e.into_bytes() )

                }
            }

            ("/register", Method::GET) => {

                let (pub_key, priv_key) = verify::create_key();
                let data = format!("{:x}:{:x}", pub_key.to_bytes(), priv_key.to_bytes());
                
                ("200 OK", data.into_bytes())
            }
            
            ("/register", Method::POST) => {

                let data_string = String::from_utf8(http.data.to_vec()).unwrap_or_default();
                let parts: Vec<&str> = data_string.split(':').collect();

                if parts.len() != 5 {
                    return Self::respond("400 Bad Request", "Invalid data format".into(), hmap!{"Content-Type"=>"text/plain"})
                }
    
                let pub_key = parts[0];
                let username = parts[1];
                let bio = parts[2];
                let pp = parts[3];
                let sign = parts[4];
                let hash = digest(format!("{}:{}:{}:{}",pub_key,username,bio,pp));


                match verify::verify(pub_key, &hash, sign) {

                    Ok(valid) => {
                        
                        if valid {
                            match database::register(pub_key.to_string()) {
                                Ok(()) => ("200 OK", "User registered successfully".into()),
                                Err(e) => ("500 Internal Server Error", e.into())
                            }
                        } else {
                            ("401 Unauthorized", "Invalid signature".into())
                        }
                    }
                    Err(e) => ("500 Internal Server Error", e.into())
                }
            }

            ("/posts", Method::GET) => {

                if let (Some(subject),Some(time)) = (http.params.get("sub"), http.params.get("t")){

                    let default_post_num = 10;
                    let post_num = http.params.get("n")
                    .map(|n| n.parse::<u8>().unwrap_or(default_post_num))
                    .unwrap_or(default_post_num);

                    if let Some(posts) = database::get_posts(subject, time, post_num){

                        ("200 OK", posts.into())
                    }
                    else {

                        ("404 Not Found", "None".into())
                    }
                } 
                else {
                    return Self::respond("422 Unprocessable Content", "Missing paramters".into(), hmap!("Content-Type"=>"text/plain"))
                }

            }

            ("/sub_posts", Method::POST) => {

                let data_string = String::from_utf8(http.data).unwrap_or_default();
                let posts: Post = match Post::new(&data_string) {
                    Some(posts) => posts,
                    None => return Self::respond("404 Not Found", "Invalid json format for post".into(), hmap!{"Content-Type"=>"text/plain"})
                };

                match database::get_sub_posts(posts) {
                    Ok(Some(post)) => ("200 OK", post.into()),
                    Ok(None) => ("404 Not Found", "".into()),
                    Err(e) => ("500 Internal Server Error", e.into())
                }
            }
    
            ("/post", Method::POST) => {

                let data_string = String::from_utf8(http.data).unwrap_or_default();
                println!("{data_string}");
                let mut posts: Post = match Post::new(&data_string) {
                    Some(posts) => posts,
                    None => return Self::respond("404 Not Found", "Invalid json format for post".into(), hmap!{"Content-Type"=>"text/plain"})
                };
                
                let post_hash = posts.hash();
                let post: & Post = posts.last();
                let pub_key: &str = &post.pub_key;
                let sign: &str = &post.sign;

                match database::post(posts) {
                    Ok(()) => ("200 OK", "Posted successfully".into()),
                    Err(e) => ("500 Internal Server Error", e.into())
                }
                /*
                match verify::verify(pub_key, &post_hash, sign) {

                    Ok(valid) => {
                        if valid {
                            match database::post(posts) {
                                Ok(()) => ("200 OK", "Posted successfully".into()),
                                Err(e) => ("500 Internal Server Error", e.into())
                            }
                        } 
                        else {
                            ("401 Unauthorized", "Invalid signature".into())
                        }
                    }
                    Err(e) => ("500 Internal Server Error", e.into())
                }
                 */
                
            }
            
            (path, Method::GET) if path.starts_with("/user/") => {
                

                todo!()
            }

            (path, Method::POST) if path.starts_with("/upload/") => {

                let profile_pic_re = Regex::new(r"^/upload/([a-fA-F0-9]{64})/pp\.(png|jpg)$").unwrap();
                
                match profile_pic_re.captures(path) {

                    Some(profile_pic) => {

                        match database::upload(path, http.data) {
                            Ok(()) => ("200 OK", "Image upload successful".into()),
                            Err(e) => ("500 Internal Server Error", e.into()),
                        }

                    }

                    None => return Self::respond("422 Unprocessable Content", "Make sure about file format".into(), hmap!("Content-Type"=>"text/plain")),
                }
            }

            ("/time", Method::GET) => {

                ("200 OK", database::get_time().into())
            }
    
            _ => {

                let path = format!(".{}", http.path);
                
                if path.contains("../") {
                    ("404 Not Found", "???".to_string());
                }

                if let Ok(data) = fs::read(path) {
                    ("200 OK", data)
                } else {
                    let data = Self::file_contents("404.html");
                    return Self::respond("404 Not Found", data, hmap! {"Content-Type"=>"text/html"})
                }
            }
            
        };

        let content_type = match http.path.split('.').last().unwrap_or("") {
            
            "html" | "/" | "/sleep" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "ico" => "image/x-icon",
            "svg" => "image/svg+xml",
            "json" | "/posts" | "/sub_posts" => "application/json",
            _ => "text/plain",

        };

        Self::respond(status, data, hmap!{"Content-Type"=>content_type})
    }

    fn connection(mut stream: TcpStream) {

        const HEADERS_SIZE: usize = 512;
    
        let mut header_buffer = [0; HEADERS_SIZE];
        let mut header_len = 0;
        
        loop {
            match stream.read(&mut header_buffer[header_len..]) {
                Ok(0) => {
                    
                    println!("Connection closed by peer");
                    return;
                }
                Ok(n) => {
                    header_len += n;
                    if let Some(pos) = header_buffer[..header_len].windows(4).position(|w| w == b"\r\n\r\n") {
                        header_len = pos + 4;
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading headers: {}", e);
                    return;
                }
            }
        }
        
    
        let http_request = &header_buffer[..header_len];
    
        
        let http = match HTTP::new(http_request) {
            Some(req) => req,
            None => {
                let response = Router::respond(
                    "400 Bad Request",
                    "Invalid Request".into(),
                    hmap! {"Content-Type" => "text/plain"},
                );
                stream.write_all(&response).unwrap();
                return;
            }
        };
    
        
        let content_length = http.headers.get("Content-Length")
            .and_then(|len| len.parse::<usize>().ok())
            .unwrap_or(0);
        
        let request: Vec<u8> = if HEADERS_SIZE < header_len + content_length {
            let left_to_read = header_len + content_length - HEADERS_SIZE;
            let mut body_buffer = vec![0; left_to_read];
            
            
            stream.read_exact(&mut body_buffer).unwrap();
            
            
            let mut complete_request = Vec::from(&header_buffer);
            complete_request.extend_from_slice(&body_buffer);
            println!("{:?}",complete_request);
            complete_request
        } else {
            Vec::from(&header_buffer[..header_len+content_length])
        };

        let response = if let Some(http) = HTTP::new(&request) {
            Router::route(http)
        } else {
            Router::respond("400 Bad Request", "".into(), hmap! {"Content-Type" => "text/plain"})
        };
    
        
        stream.write_all(&response).unwrap();
    }
        
    
}