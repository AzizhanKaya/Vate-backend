#[warn(unreachable_code)]

use std::fs;
use std::io::{Write, BufRead, BufReader, Read};
use std::path::Path;
use chrono::Utc;
use sha256::digest;
use std::fs::{File, OpenOptions};
use serde_json::json;

pub fn get_time() -> String {

    let now = Utc::now();
    let seconds = now.timestamp();

    seconds.to_string()
}


pub fn check_userlist(file: &File, pub_key: &str) -> Option<()> {
    
    let reader = BufReader::new(file);

    for line in reader.lines() {
        match line {
            Ok(line) if line == pub_key => return None,
            Err(_) => return None,
            _ => continue,
        }
    }

    Some(())
}

pub fn register(pub_key: &str) -> Result<(), String> {

    let dir = format!("./{}", pub_key);
    let userlist_path = "user.list";

    fs::create_dir_all(&dir).map_err(|_| "Failed to create user directory".to_string())?;

    if !Path::new(userlist_path).exists() {
        File::create(userlist_path).map_err(|_| "Failed to create user.list file".to_string())?;
    }

    let file = File::open(userlist_path).map_err(|_| "Failed to open user.list".to_string())?;

    let user_exists = check_userlist(&file, pub_key).is_none();

    if user_exists {
        return Err("User already exists".to_string());
    }

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(userlist_path)
        .map_err(|_| "Failed to open user.list for writing".to_string())?;
    writeln!(file, "{}", pub_key).map_err(|_| "Failed to write to user.list".to_string())?;

    Ok(())
}


pub fn post(pub_key: &str, subject: &str, message: &str, time: &str, sign: &str, hash: &str) -> Result<(), String> {

    let timestamp = get_time();

    let server_time = timestamp.parse::<u64>().map_err(|_| "Failed to parse current timestamp".to_string())?;
    let client_time = time.parse::<u64>().map_err(|_| "Failed to parse provided time".to_string())?;

    if server_time.abs_diff(client_time) > 30 {
        return Err(format!("Time is not synchronized: {server_time}"));
    }

    let dir_path = Path::new(pub_key);

    if !dir_path.exists() {
        return Err("User has not registered".to_string());
    }

    

    let mut posts = vec![];

    for entry in fs::read_dir(dir_path).map_err(|e| e.to_string())? {

        let entry = entry.map_err(|e| e.to_string())?;
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        if file_name_str.ends_with(".post") {
            posts.push(file_name_str.to_string());
        }
    }

    posts.sort();
    let last_post = posts.last();

    let post_number = if let Some(last_post) = posts.last() {

        let parts: Vec<&str> = last_post.split('.').collect();
        if let Ok(number) = parts[0].parse::<u32>() {
            number + 1
        } else {
            return Err("Failed to parse post number from the last post".to_string());
        }
    } else {
        1
    };

    let (past_hash, next_hash) = if let Some(last_post) = last_post {

        let last_post_path = dir_path.join(last_post);
        let mut file = File::open(last_post_path).map_err(|e| e.to_string())?;
        let mut last_post_content = String::new();
        file.read_to_string(&mut last_post_content).map_err(|e| e.to_string())?;

        let last_next_hash = last_post_content
            .lines()
            .last()
            .ok_or("Failed to read next_hash from last post".to_string())?;

        let next_hash = digest(format!("{last_next_hash}:{hash}"));

        (last_next_hash.to_string(), next_hash)  
    }
    else {

        let next_hash = digest(format!("{hash}:{hash}"));
        (hash.to_string(), next_hash)
    };

    let file_name = format!("{post_number}.{subject}.{timestamp}.post");
    let file_path = dir_path.join(file_name);

    let mut file = File::create(file_path).map_err(|e| e.to_string())?;
    let content = format!("{past_hash}\n\n{pub_key}:{message}:{sign}\n\n{next_hash}");
    file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn get_posts(subject: &str, time: &str, post_num: u8) -> Option<String> {

    let mut posts = Vec::new();

    for user in fs::read_dir(".").unwrap() {
        let user = user.unwrap();
        let user_folder = user.file_name();
        let user_folder_name = user_folder.to_string_lossy();

        if !user_folder_name.chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }

        if user.path().is_dir() {

            let files = fs::read_dir(user.path()).unwrap();

            for file in files {

                let file = file.unwrap();
                let file_name = file.file_name();
                let file_name = file_name.to_string_lossy();

                let parts: Vec<&str> = file_name.split('.').collect();

                if parts.len() != 4 || *parts.last().unwrap() != "post" {
                    continue;
                }

                let post_subject = parts[1];
                let post_time = parts[2];

                if post_subject == subject && post_time < time {
                    posts.push(file.path().to_string_lossy().to_string());
                }

                
            }
        }

        if posts.len() >= post_num.into() {
            break;
        }
    }

    if posts.is_empty() {
        return None;
    }

    
    
    posts.sort_by(|a, b| {
        let time_a: u64 = a.split('.').nth(3).unwrap().parse().unwrap();
        let time_b: u64 = b.split('.').nth(3).unwrap().parse().unwrap();
        time_b.cmp(&time_a)
    });

    let post_count = posts.len().min(post_num as usize);
    posts = posts[..post_count].to_vec();


    let mut posts_json = Vec::new();

    for post_path in posts {

        let contents = fs::read_to_string(&post_path).ok()?;
        let mut lines = contents.lines();

        let past_hash = lines.next()?;
        let content = lines.next()?;
        let next_hash = lines.last()?;

        let parts: Vec<&str> = Path::new(&post_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .split('.')
            .collect();

        let content_parts: Vec<&str> = content.split(':').collect();
        if content_parts.len() != 3 {
            continue;
        }
        let (pub_key, message, sign) = (content_parts[0], content_parts[1], content_parts[2]);

        let post_json = json!({
            subject: parts[1],
            time: parts[2],
            content: {
                pub_key: pub_key,
                message: message,
                "sign": sign,
            }
        });

        posts_json.push(post_json);
    }

    Some(json!(posts_json).to_string())
}



fn backup() {
    todo!()
}