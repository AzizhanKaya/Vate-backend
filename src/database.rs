#[warn(unreachable_code)]
#[warn(while_true)]
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Read, Write};
use std::path::Path;
use chrono::Utc;
use sha256::digest;
use std::fs::{File, OpenOptions};
use serde_json::json;

use crate::Post;

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

pub fn register(pub_key: String) -> Result<(), String> {

    let dir = format!("./{}", pub_key);
    let userlist_path = "user.list";

    fs::create_dir_all(&dir).map_err(|_| "Failed to create user directory".to_string())?;

    if !Path::new(userlist_path).exists() {
        File::create(userlist_path).map_err(|_| "Failed to create user.list file".to_string())?;
    }

    let file = File::open(userlist_path).map_err(|_| "Failed to open user.list".to_string())?;

    let user_exists = check_userlist(&file, &pub_key).is_none();

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


pub fn post(post: Post) -> Result<(), String> {

    let timestamp = get_time();

    let server_time = timestamp.parse::<u64>().map_err(|_| "Failed to parse current timestamp".to_string())?;
    let client_time = post.last().time.parse::<u64>().map_err(|_| "Failed to parse provided time".to_string())?;

    if server_time.abs_diff(client_time) > 1000 {
        return Err(format!("Time is not synchronized: {server_time}"));
    }

    let mut dir_path = Path::new(&post.pub_key);

    if !dir_path.exists() {
        return Err("User has not registered".to_string());
    }

    dir_path = Path::new(&post.pub_key);

    if !dir_path.exists() {
        return Err("User not found".to_string());
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

    let post_number = if let Some(last_post) = last_post {

        let parts: Vec<&str> = last_post.split('.').collect();
        if let Ok(number) = parts[0].parse::<u32>() {
            number
        } else {
            return Err("Failed to parse post number from the last post".to_string());
        }
    } else {

        if post.post.is_some(){
            return Err("Post does not exists".to_string());
        }

        1
    };

    match post.post {

        None => {
            let (past_hash, next_hash) = if let Some(last_post) = last_post {

                let last_post_path = dir_path.join(last_post);
                let mut file = File::open(last_post_path).map_err(|e| e.to_string())?;
                let mut last_post_content = String::new();
                file.read_to_string(&mut last_post_content).map_err(|e| e.to_string())?;
    
                let last_next_hash = last_post_content
                    .lines()
                    .last()
                    .ok_or("Failed to read next_hash from last post".to_string())?
                    .to_string();
    
                let next_hash = post.hash(last_next_hash.clone());
    
                (last_next_hash, next_hash)
            }
            else {
                let past_hash = digest(format!("{}:{}:{}:{}", 
                    post.pub_key,
                    post.subject,
                    post.message,
                    post.time
                ));
    
                let next_hash = post.hash(past_hash.clone());
                (past_hash, next_hash)
            };
    
            let file_name = format!("{}.{}.{}.post",post_number+1, post.subject, post.time);
            let file_path = dir_path.join(file_name);
    
            let mut file = File::create(file_path).map_err(|e| e.to_string())?;
            let content = format!("{past_hash}\n\n{}:{}:{}:{}:{}\n\n{next_hash}", post.pub_key, post.subject ,post.message, post.time, post.sign);
            file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
            
            Ok(())
        }

        Some(_) => {
        
        let file_name = format!("{}.{}.{}.post", post_number, post.subject, post.time);
        let file_path = dir_path.join(file_name);
        let mut file = OpenOptions::new().read(true).write(true).open(&file_path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(&file);

        let mut current_post = &post;
        let mut lines = reader.lines();
        
        let mut expected_line = format!(
            "{}:{}:{}:{}:{}",
            current_post.pub_key,
            current_post.subject,
            current_post.message,
            current_post.time,
            current_post.sign
        );

        let mut i = 0;
        let mut position = 0;
        let mut found = false;
        let mut level = 0;

        while let Some(line) = lines.next() {


            let line = line.map_err(|e| e.to_string())?;
            level = line.chars().take_while(|&c| c == ' ').count();

            position += line.len() + 1;

            if level == i {
                
                if line.trim() == expected_line {

                    if let Some(ref next_post) = current_post.post {
                        current_post = next_post;
                        i+=1;
                        expected_line = format!(
                            "{}:{}:{}:{}:{}",
                            current_post.pub_key,
                            current_post.subject,
                            current_post.message,
                            current_post.time,
                            current_post.sign
                        );
                    } 
                    if current_post.post.is_none(){
                        found = true;
                        level+=1;
                        break;
                    }
                }
            }
        }

        if found {
              
        file.seek(SeekFrom::Start(position as u64));

        
        let mut remainder = Vec::new();
        file.read_to_end(&mut remainder);

        file.seek(SeekFrom::Start(position as u64));

        let data = format!("{}{}\n", " ".repeat(level), expected_line);

        file.write_all(data.as_bytes());

        file.write_all(&remainder);

        }
        
        Ok(())
        }
    }
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
            "subject": parts[1],
            "time": parts[2],
            "content": {
                "pub_key": pub_key,
                "message": message,
                "sign": sign,
            }
        });

        posts_json.push(post_json);
    }

    Some(json!(posts_json).to_string())
}

pub fn like(pub_key: &str, content: &str,sign: &str, hash: &str) -> Result<(), String> {



    todo!()
}



fn backup() {
    todo!()
}