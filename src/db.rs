use turso::Builder;
use markdown::{to_html};

pub async fn create_table() {
    let db = Builder::new_local("wikis.db").build().await.expect("It should be possible to create a local database");
    let conn = db.connect().expect("It should be possible to connect to a local database");

    // Create a table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS wikis (id INTEGER PRIMARY KEY, user TEXT, content TEXT)",
        ()
    ).await.expect("It should be possible to create a table within the database");
}

pub async fn get_record(username: &str) -> Option<String> {
    let db = Builder::new_local("wikis.db").build().await.ok()?;
    let conn = db.connect().ok()?;

    let mut rows = conn.query("SELECT content FROM wikis WHERE user = ?", (username,)).await.ok()?;

    if let Some(row) = rows.next().await.ok()? {
        let content: String = row.get(0).ok()?;
        return Some(content);
    }
    
    None
}
pub async fn insert_record(markdown_text: &str, username: &str) -> Option<String> {
    let db = Builder::new_local("wikis.db").build().await.ok()?;
    let conn = db.connect().ok()?;
    let html_text = to_html(markdown_text);
    if html_text != markdown_text { // conversion happened correctly
        let user_exists = get_record(username).await;
        match user_exists {
            Some(_) => {
                return Some("User already exists".to_string())
            }
            None => {
                conn.execute(
                    "INSERT INTO wikis (user, content) VALUES (?1, ?2)", 
                    ["Bob", "bob@example.com"]
                ).await.ok()?;
                return None
            }
        }
    }
    Some("Could not convert markdown text to HTML".to_string())
}