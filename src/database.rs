#![allow(unreachable_code)]
#![allow(while_true)]
#![allow(unused_must_use)]

use std::fs;
use std::io::{BufRead, BufReader, Seek, SeekFrom, Read, Write};
use std::path::Path;
use chrono::Utc;
use sha256::digest;
use std::fs::{File, OpenOptions};
use serde_json::json;
use serde_json::Value;

use crate::Post;

pub fn get_time() -> u64 {

    let now = Utc::now();
    let seconds = now.timestamp();

    seconds as u64
}


pub fn check_userlist(file: &File, pub_key: &str) -> Option<()> {
    let reader = BufReader::new(file);

    for line in reader.lines() {
        
        match line.unwrap().split(':').nth(0) {
            Some(key) if key == pub_key => return Some(()),
            Some(_) | None => continue,
        }
    }

    None
}


pub fn register(pub_key: &str, username: &str, bio: &str, sign: &str) -> Result<(), &'static str> {

    let timestamp = get_time();

    let userlist_path = "user.list";

    let file = File::open(userlist_path).map_err(|_| "Failed to open user.list")?;

    let user_exists = check_userlist(&file, pub_key).is_some();

    if user_exists {
        return Err("User already exists");
    }

    let dir = format!("./{}", pub_key);

    fs::create_dir_all(&dir).map_err(|_| "Failed to create user directory")?;

    if !Path::new(userlist_path).exists() {
        File::create(userlist_path).map_err(|_| "Failed to create user.list file")?;
    }

    let info_path = format!("{}/info", dir);
    let mut info_file = File::create(&info_path).map_err(|_| "Failed to create info file")?;
    writeln!(info_file, "{}:{}:{}", username, bio, timestamp).map_err(|_| "Failed to write to info file")?;


    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(userlist_path)
        .map_err(|_| "Failed to open user.list for writing")?;
    writeln!(file, "{}:{}:{}", pub_key, username, sign).map_err(|_| "Failed to write to user.list")?;
    
    Ok(())
}


pub fn post(mut post: Post) -> Result<(), String> {

    let server_time = get_time();
    let client_time = post.last().time.parse::<u64>().map_err(|_| "Failed to parse provided time".to_string())?;

    if server_time.abs_diff(client_time) > 1000 {
        return Err(format!("Time is not synchronized: {server_time}"));
    }

    let path_binding = post.pub_key.clone();
    let dir_path = Path::new(&path_binding);

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

    posts.sort_by(|a, b| {
        let num_a: u64 = a.split('.').nth(0).unwrap().parse().unwrap();
        let num_b: u64 = b.split('.').nth(0).unwrap().parse().unwrap();
        num_a.cmp(&num_b)
    });

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

        0
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

                if last_next_hash != post.past_hash.clone().unwrap() {
                    return Err("Past hashes does not match".to_string());
                }
    
                let next_hash = post.hash();
    
                (last_next_hash, next_hash)
            }
            else {
                let past_hash = digest(format!("{}:{}:{}:{}", 
                    post.pub_key,
                    post.subject,
                    post.message,
                    post.time
                ));
                post.past_hash = Some(past_hash.clone());
                let next_hash = post.hash();
                (past_hash, next_hash)
            };
    
            let file_name = format!("{}.{}.{}.post",post_number + 1, post.subject, post.time);
            let file_path = dir_path.join(file_name);
    
            let mut file = File::create(file_path).map_err(|e| e.to_string())?;
            let content = format!("{past_hash}\n\n{}:{}:{}:{}:{}\n\n{next_hash}", post.pub_key, post.subject ,post.message, post.time, post.sign);
            file.write_all(content.as_bytes()).map_err(|e| e.to_string())?;
            
            Ok(())
        }

        Some(_) => {
        
        let file_name: String = posts.into_iter().filter(|file_name| {
            file_name.ends_with(&format!(".{}.{}.post", post.subject, post.time))
        }).nth(0).unwrap();

        let file_path = dir_path.join(file_name);
        let mut file = OpenOptions::new().read(true).write(true).open(&file_path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(&file);

        let mut post_iter = post.iter();
        let mut current_post = post_iter.next().unwrap();
        let mut lines = reader.lines();
        
        let mut expected_line = current_post.format();

        let mut i = 0;
        let mut position = 0;
        let mut found = false;
        let mut level = 0;

        while let Some(line) = lines.next() {

            let line = line.map_err(|e| e.to_string())?;
            level = line.chars().take_while(|&c| c == ' ').count();

            position += line.len() + 1;

            if level < i {
                break;
            }

            if level == i && line.trim() == expected_line {

                current_post = post_iter.next().unwrap();
                expected_line = current_post.format();

                match &current_post.post {
                    
                    Some(post) => {
                        i+=1
                    }

                    None => {
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

            let data = format!("{}{}\n", " ".repeat(level), current_post.format());

            file.write_all(data.as_bytes());

            file.write_all(&remainder);

            return Ok(());
        }
        
        Err("Couldn't find the post-chain".to_string())
        }
    }
}

fn get_likes_and_posts_count(post_path: &str, post: &Post) -> Result<(u64, u64), String> {
    
    let file = OpenOptions::new().read(true).write(false).open(&post_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(&file);
    
    let mut post_iter = post.iter();
    let mut current_post = post_iter.next().unwrap();
    let mut lines = reader.lines();
    
    let mut expected_line = current_post.format();

    let mut i = 0;
    let mut found = false;
    let mut level;

    while let Some(line) = lines.next() {

        let line = line.map_err(|e| e.to_string())?;
        level = line.chars().take_while(|&c| c == ' ').count();


        if level < i {
            break;
        }

        if level == i && line.trim() == expected_line {


            match &current_post.post {
                
                Some(post) => {
                    i+=1
                }

                None => {
                    found = true;
                    i+=1;
                    break;
                }
            }

            current_post = post_iter.next().unwrap();
            expected_line = current_post.format();
            
        }
    }

    let expected_message = "&L";
    let mut likecount: u64 = 0;
    let mut posts_len: u64 = 0;

    if found {
              
        while let Some(line) = lines.next(){

            let line = line.map_err(|e| e.to_string())?;
            level = line.chars().take_while(|&c| c == ' ').count();

            let message: &str = line.trim().split(':').nth(2).unwrap_or("");

            if level < i {
                break;
            }

            if level == i {
                if message == expected_message{
                    likecount += 1;
                } else {
                    posts_len += 1;
                }
            }
        }

    }

    Ok((likecount, posts_len))
}

pub fn user_posts(pub_key: &str) -> Option<String> {
    
    let user_directory = format!("./{}", pub_key);

   
    if !Path::new(&user_directory).exists() {
        return None;
    }

    let file = File::open("user.list").unwrap();

    if check_userlist(&file, pub_key).is_none() {
        return None;
    }

    
    let mut posts_json = Vec::new();

    
    for entry in fs::read_dir(&user_directory).ok()? {
        let entry = entry.ok()?;
        let file_path = entry.path();


        let user = user(pub_key).ok()?;
        let account: Value = serde_json::from_str(&user).expect("Json parsing error");

        if file_path.extension().and_then(|ext| ext.to_str()) == Some("post") {
            
            let file = File::open(&file_path).ok()?;
            let reader = BufReader::new(file);

            
            let mut lines = reader.lines();

            
            let past_hash = lines.next().and_then(|line| line.ok())?;
            lines.next();
            let content = lines.next().and_then(|line| line.ok())?;
            lines.next();
            let next_hash = lines.next().and_then(|line| line.ok())?;

            
            let content_parts: Vec<&str> = content.split(':').collect();
            if content_parts.len() != 5 {
                continue;
            }
            let (post_pub_key, subject, message, time, sign) = (
                content_parts[0],
                content_parts[1],
                content_parts[2],
                content_parts[3],
                content_parts[4],
            );

            
            let post: Post = Post::new(&format!(r#"
            {{
                "past_hash": "{past_hash}",
                "pub_key": "{pub_key}",
                "subject": "{subject}",
                "message": "{message}",
                "time": "{time}",
                "sign": "{sign}"
            }}"#)).unwrap();

            let (likes, posts_len) = get_likes_and_posts_count(&file_path.to_string_lossy(), &post).unwrap();

            let post_json = json!({
                "account":{
                    "img_type":account["img_type"],
                    "username":account["username"],
                    "pub_key": pub_key,
                },
                "past_hash": past_hash,
                "subject": subject,
                "message": message,
                "time": time,
                "sign": sign,
                "likes": likes,
                "posts": posts_len
            });

            
            posts_json.push(post_json);
        }
    }
    
    posts_json.sort_by(|a, b| {
        let time_a = a["time"].as_str().and_then(|s| s.parse::<u64>().ok()).unwrap();
        let time_b = b["time"].as_str().and_then(|s| s.parse::<u64>().ok()).unwrap();
        time_b.cmp(&time_a)
    });
    

    if posts_json.is_empty() {
        None
    } else {
        Some(json!(posts_json).to_string())
    }
}

pub fn get_posts(subject: &str, time: &str,post_num: u8, direction: &str) -> Option<String> {

    let mut posts = Vec::new();

    let direction= direction.parse::<bool>().ok()?;

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

                if post_subject == subject && (post_time <= time) ^ direction {
                    posts.push(file.path().to_string_lossy().to_string());
                }

                
            }
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
        lines.next()?;
        let content = lines.next()?;
        lines.next()?;
        let next_hash = lines.last()?;

        let parts: Vec<&str> = post_path.split('.').collect();

        let content_parts: Vec<&str> = content.split(':').collect();
        if content_parts.len() != 5 {
            continue;
        }
        let (pub_key, subject, message, time, sign) = (content_parts[0], content_parts[1],content_parts[2], content_parts[3], content_parts[4]);

        let post: Post = Post::new(&format!(r#"
        {{
            "past_hash": "{past_hash}",
            "pub_key": "{pub_key}",
            "subject": "{subject}",
            "message": "{message}",
            "time": "{time}",
            "sign": "{sign}"
        }}"#)).unwrap();

        let (likes, posts_len) = get_likes_and_posts_count(&post_path, &post).unwrap();

        let user = user(pub_key).ok()?;

        let account: Value = serde_json::from_str(&user).expect("Json parsing error");

        let post_json = json!({
            "account":{
                "img_type":account["img_type"],
                "username":account["username"],
                "pub_key": pub_key,
            },
            "past_hash": past_hash,
            "subject": subject,
            "message": message,
            "time": time,
            "sign": sign,
            "likes": likes,
            "posts": posts_len
        });

        posts_json.push(post_json);
    }

    

    Some(json!(posts_json).to_string())
}

pub fn get_sub_posts(post: Post) -> Result<Option<String>, String> {

    let user_path = Path::new(&post.pub_key);
    let user_dir = fs::read_dir(&post.pub_key).map_err(|e| e.to_string())?;

    
    let mut post_name = String::new();

    for file in user_dir {

        let file = file.unwrap();
        let file_name = file.file_name();
        let file_name = file_name.to_string_lossy();

        let parts: Vec<&str> = file_name.split('.').collect();

        if parts.len() != 4 || *parts.last().unwrap() != "post" {
            continue;
        }
        let post_number = parts[0];
        let post_subject = parts[1];
        let post_time = parts[2];

        if post_subject == post.subject && post_time == post.time {
            post_name = format!("{}.{}.{}.post", post_number, post_subject, post_time);
            break;
        }
    }

    if post_name.is_empty(){
        return Err("Couldn't find the post".to_string());
    }

    let post_path = user_path.join(post_name);
    let file = OpenOptions::new().read(true).write(false).open(&post_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(&file);

    let mut post_iter = post.iter();
    let mut current_post = post_iter.next().unwrap();
    
    let mut expected_line = current_post.format();

    let mut lines = reader.lines();
    let mut i = 0;
    let mut found = false;
    let mut level ;
    
    while let Some(line) = lines.next() {

        let line = line.map_err(|e| e.to_string())?;
        level = line.chars().take_while(|&c| c == ' ').count();

        if level == i && line.trim() == expected_line {

            i+=1;
            match &current_post.post {
                    
                Some(post) => {
                    current_post = post_iter.next().unwrap();
                    expected_line = current_post.format();
                }

                None => {
                    found = true;
                    break;
                }
            }

            
        }
    }

    let mut posts_json = Vec::new();

    if found {

        let mut post_mut = post.clone();

        while let Some(line) = lines.next()  {

            let line = line.map_err(|e| e.to_string())?;
            level = line.chars().take_while(|&c| c == ' ').count();

            if level < i {
                break;
            }

            if level == i {

                let parts: Vec<&str> = line.split(':').collect();

                let (pub_key, subject, message, time,sign) = (parts[0].trim(), parts[1], parts[2], parts[3], parts[4].trim());
                
                let sub_post: Box<Post> = Box::new(Post::new(&format!(
                r#"
                {{
                    "pub_key": "{pub_key}",
                    "subject": "{subject}",
                    "message": "{message}",
                    "time": "{time}",
                    "sign": "{sign}"
                }}"#
                )).unwrap());

                post_mut.post = Some(sub_post);

                let (likes, posts_len) = get_likes_and_posts_count(&post_path.to_string_lossy(), &post_mut).unwrap();
                
                let user = user(pub_key).unwrap();
                let account: Value = serde_json::from_str(&user).expect("Json parsing error");

                let post_json = json!({
                    "account":{
                        "img_type":account["img_type"],
                        "username":account["username"],
                        "pub_key": pub_key,
                    },
                    "subject": subject,
                    "message": message,
                    "time": time,
                    "sign": sign,
                    "likes": likes,
                    "posts": posts_len
                });
        
                posts_json.push(post_json);
            }
        }
    }

    if !posts_json.is_empty(){
        return Ok(Some(json!(posts_json).to_string()));
    } else {
        return Ok(None);
    }

}

pub fn like(pub_key: &str, content: &str,sign: &str, hash: &str) -> Result<(), String> {



    todo!()
}

pub fn upload_profile_pic(path: &str, data: Vec<u8>) -> Result<(), String> {
    
    let user = path.split('/').nth(2).ok_or("User has not found")?;
    
    
    let user_directory = format!("./{}", user);

    
    if !Path::new(&user_directory).exists() {
        return Err("User does not exist.".to_string());
    }

    let file_format = path.split('/').nth(3).unwrap().split('.').nth(1).ok_or("File format has not found")?;
    let pic_path = format!("./{}/pp.{}", user_directory, file_format);

    
    let mut file = File::create(&pic_path).map_err(|e| e.to_string())?;
    file.write_all(&data).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn user(user: &str) -> Result<String, String> {

    

    let file = File::open("user.list").map_err(|_| "Failed to open user.list")?;

    if check_userlist(&file,user).is_none() {
        return Err("User does not exist.".to_string());
    }

    let user_directory = format!("./{}", user);

    if !Path::new(&user_directory).exists() {
        return Err("User does not exist.".to_string());
    }

    fn img_ext(user_directory: &str) -> Option<String> {
        let img_type = fs::read_dir(user_directory)
            .ok()?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.path().is_file() && matches!(
                    entry.path().extension().and_then(|ext| ext.to_str()),
                    Some("png") | Some("jpeg") | Some("jpg")
                )
            })
            .filter_map(|entry| {
                entry.path().extension().and_then(|ext| {
                    let extension_str = ext.to_string_lossy().into_owned();
                    Some(extension_str)
                })
            })
            .next();
    
        img_type 
    }

    let img_type = img_ext(&user_directory);

    let user_info = format!("./{}/info", user);
    let file = File::open(&user_info).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    if let Some(Ok(line)) = reader.lines().next() {
        
        let parts: Vec<&str> = line.split(':').collect();
        let username = parts[0];
        let bio = parts[1];
        let time = parts[2];

        let info_json = json!({
            "username": username,
            "bio": bio,
            "time": time,
            "img_type": img_type,
            "pub_key": user
        });

        return Ok(info_json.to_string());

    } else {
        return Err("No info found about user".to_string());
    }

}



fn backup() {
    todo!()
}