use turso::Builder;
use markdown::{to_html};
use serde::{Deserialize, Serialize};
use axum::{Router, routing::{get, post},extract::Path, response::{Json,Html}};

const CSS_STYLE: &str = "<style>
  * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, 'Apple Color Emoji', Arial, sans-serif, 'Segoe UI Emoji', 'Segoe UI Symbol';
    font-size: 16px;
    line-height: 1.6;
    color: #37352f;
    background: #ffffff;
    padding: 0;
    margin: 0;
  }

  /* Main content container */
  body > * {
    max-width: 720px;
    margin: 0 auto;
    padding: 40px 96px;
  }

  /* Headings */
  h1 {
    font-size: 2.5em;
    font-weight: 700;
    line-height: 1.2;
    margin: 2px 0 8px;
    color: #37352f;
  }

  h2 {
    font-size: 1.875em;
    font-weight: 600;
    line-height: 1.3;
    margin: 1.4em 0 4px;
    color: #37352f;
  }

  h3 {
    font-size: 1.5em;
    font-weight: 600;
    line-height: 1.3;
    margin: 1em 0 4px;
    color: #37352f;
  }

  h4, h5, h6 {
    font-size: 1.25em;
    font-weight: 600;
    line-height: 1.4;
    margin: 1em 0 4px;
    color: #37352f;
  }

  /* Paragraphs */
  p {
    margin: 4px 0;
    padding: 3px 2px;
    white-space: pre-wrap;
  }

  /* Links */
  a {
    color: inherit;
    text-decoration: underline;
    text-decoration-color: rgba(55, 53, 47, 0.4);
    text-underline-offset: 2px;
    transition: text-decoration-color 0.2s ease;
  }

  a:hover {
    text-decoration-color: rgba(55, 53, 47, 0.8);
  }

  /* Lists */
  ul, ol {
    margin: 4px 0;
    padding-left: 1.5em;
  }

  li {
    margin: 2px 0;
    padding: 3px 2px;
  }

  li::marker {
    color: rgba(55, 53, 47, 0.6);
  }

  /* Code */
  code {
    background: rgba(135, 131, 120, 0.15);
    color: #eb5757;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-size: 0.85em;
    font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, Courier, monospace;
  }

  pre {
    background: #f7f6f3;
    padding: 16px;
    border-radius: 3px;
    overflow-x: auto;
    margin: 8px 0;
  }

  pre code {
    background: none;
    color: #37352f;
    padding: 0;
    font-size: 0.875em;
  }

  /* Blockquotes */
  blockquote {
    border-left: 3px solid #37352f;
    padding-left: 16px;
    margin: 8px 0;
    color: #37352f;
  }

  /* Horizontal rule */
  hr {
    border: none;
    border-top: 1px solid rgba(55, 53, 47, 0.16);
    margin: 16px 0;
  }

  /* Images */
  img {
    max-width: 100%;
    height: auto;
    border-radius: 3px;
    margin: 8px 0;
  }

  /* Tables */
  table {
    border-collapse: collapse;
    width: 100%;
    margin: 8px 0;
  }

  th, td {
    border: 1px solid rgba(55, 53, 47, 0.16);
    padding: 8px 12px;
    text-align: left;
  }

  th {
    background: rgba(242, 241, 238, 0.6);
    font-weight: 600;
  }

  /* Mobile responsiveness */
  @media (max-width: 768px) {
    body > * {
      padding: 40px 24px;
    }
  }
</style>";

async fn create_table() {
    let db = Builder::new_local("wikis.db").build().await.expect("It should be possible to create a local database");
    let conn = db.connect().expect("It should be possible to connect to a local database");

    // Create a table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS wikis (id INTEGER PRIMARY KEY, user TEXT, content TEXT)",
        ()
    ).await.expect("It should be possible to create a table within the database");
}

async fn get_record(username: &str) -> Option<String> {
    create_table().await;
    let db = Builder::new_local("wikis.db").build().await.ok()?;
    let conn = db.connect().ok()?;

    let mut rows = conn.query("SELECT content FROM wikis WHERE user = ?", (username,)).await.ok()?;

    if let Some(row) = rows.next().await.ok()? {
        let content: String = row.get(0).ok()?;
        return Some(content);
    }
    
    None
}
async fn insert_record(markdown_text: &str, username: &str) -> Option<String> {
    create_table().await;
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
                    [username, markdown_text]
                ).await.ok()?;
                return None
            }
        }
    }
    Some("Could not convert markdown text to HTML".to_string())
}
async fn update_record(markdown_text: &str, username: &str) -> Option<String> {
    create_table().await;
    let db = Builder::new_local("wikis.db").build().await.ok()?;
    let conn = db.connect().ok()?;
    let html_text = to_html(markdown_text);
    if html_text != markdown_text { // conversion happened correctly
        let user_exists = get_record(username).await;
        match user_exists {
            Some(_) => {
                conn.execute(
                    "UPDATE wikis SET content = ?1 WHERE user = ?2", 
                    [markdown_text, username]
                ).await.ok()?;
                return None
            }
            None => {
                return Some("User does not exists".to_string())
            }
        }
    }
    Some("Could not convert markdown text to HTML".to_string())
}

#[derive(Deserialize)]
struct CreateOrUpdateWikiRequest {
    content: String,
    username: String
}

#[derive(Serialize)]
struct CreateOrUpdateWikiResponse {
    success: bool,
    error: Option<String>,
    url: Option<String>
}

impl CreateOrUpdateWikiResponse {
  fn new(success: bool, error: Option<String>, url: Option<String>) -> Self {
    Self {
      success: success,
      error: error,
      url: url
    }
  }
}


async fn create_wiki(Json(payload): Json<CreateOrUpdateWikiRequest>) -> Json<CreateOrUpdateWikiResponse> {
    if let Some(error_msg) = insert_record(&payload.content, &payload.username).await {
        return Json(CreateOrUpdateWikiResponse::new(false, Some(error_msg), None))
    }
    Json(CreateOrUpdateWikiResponse::new(true, None, Some(format!("/wikis/{}", &payload.username))))
}

async fn update_wiki(Json(payload): Json<CreateOrUpdateWikiRequest>) -> Json<CreateOrUpdateWikiResponse> {
    if let Some(error_msg) = update_record(&payload.content, &payload.username).await {
        return Json(CreateOrUpdateWikiResponse::new(false, Some(error_msg), None))
    }
    Json(CreateOrUpdateWikiResponse::new(true, None, Some(format!("/wikis/{}", &payload.username))))
}

fn style_html(html: &str, username: &str) -> String {
    format!("<html>\n<head>\n<title>{}'s Wiki</title>\n{}\n</head>\n<body>\n{}\n</body>\n</html>", username, CSS_STYLE, html)
}

async fn get_wiki(Path(username): Path<String>) -> Html<String> {
    match get_record(&username).await {
        Some(content) => {
            let styled_content = style_html(&content, &username);
            return Html(styled_content)
        }
        ,
        None => {
          return Html(format!("Wiki for user {} not found... Please create one and try again!", &username))
        }
    }
}

#[tokio::main]
async fn main() {
  let app = Router::new()
    .route("/", get(|| async { "Hello, World!" }))
    .route("/wikis", post(create_wiki).patch(update_wiki))
    .route("/wikis/{username}", get(get_wiki));
  let address = "0.0.0.0:3000";
  let listener = tokio::net::TcpListener::bind(address).await.unwrap();
  println!("Starting to serving application on {}", address);
  axum::serve(listener, app).await.unwrap();
}
