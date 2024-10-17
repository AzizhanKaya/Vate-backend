use crate::Post;
use sha256::digest;
#[test]
fn test_post() {
    
    let post_data = r#"
    {
        "past_hash": "adf9255e58b91201222c7e560afae34df",
        "pub_key": "9bb40052e7eac3079fe935c8048ecfdf",
        "subject": "Test subject",
        "message": "Test Message",
        "time": "123456789",
        "sign": "efa2ea8aecae9d4e33cbbc471e80782b"
    }"#;

    
    let mut post = Post::new(post_data).expect("Failed to create Post from JSON");

    
    assert_eq!(post.subject, "Test subject");
    assert_eq!(post.message, "Test Message");
    assert_eq!(post.lenght(), 1);

    let hash: String = digest(format!("{}:{}:{}:{}:{}", 
            post.past_hash.as_ref().unwrap(),
            post.pub_key,
            post.subject,
            post.message,
            post.time
    ));
    assert_eq!(post.hash(), hash);


}

#[test]
fn test_sub_post(){

    let sub_post_data = r#"
    {
        "past_hash": "adf9255e58b91201222c7e560afae34df",
        "pub_key": "9bb40052e7eac3079fe935c8048ecfdf",
        "subject": "Test subject",
        "message": "Test Message",
        "time": "123456789",
        "sign": "efa2ea8aecae9d4e33cbbc471e80782b",
        "post": {
            "past_hash": "0a224fd3eccb8555e1d5ee2d751c4b8a",
            "pub_key": "30faae7a3b464a38eb03130c932d4a5d",
            "subject": "sub post subject",
            "message": "sub post Message",
            "time": "1234567890",
            "sign": "6012ca893b5e6bed86744c1f879330ae"
        }
    }"#;

    let mut post = Post::new(sub_post_data).expect("Failed to create Post from JSON");
    let post_hash = post.hash();
    assert_eq!(post_hash, "18597814b0e0a971931828964925eda41b3e389ac99794ff630c66bcaaa51d37");
    assert_eq!(post.last().subject, "sub post subject");
    assert_eq!(post.last().message, "sub post Message");
    assert_eq!(post.lenght(), 2);

    let mut post_iter = post.iter();

    println!("{:#?}", post_iter.nth(0))
}