use libsql::{Builder, params};
use markdown::{to_html};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde::{Deserialize, Serialize};
use axum::{Router, routing::{get, post},extract::Path, response::{Json,Html}};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{error, info, instrument};
use derivative::Derivative;

const CSS_STYLE: &str = r#"<style>
  .wiki-container * {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 1.6;
    color: #37352f;
    background: #f8f9fa;
    min-height: 100vh;
    margin: 0;
    padding: 0;
  }

  .wiki-container {
    max-width: 820px;
    margin: 0 auto;
    padding: 40px 60px;
    background: #ffffff;
    min-height: calc(100vh - 64px);
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.05);
  }

  /* Headings */
  .wiki-container h1 {
    font-size: 2.5em;
    font-weight: 700;
    line-height: 1.2;
    margin: 0 0 16px 0;
    color: #37352f;
    padding-top: 8px;
    border-bottom: 1px solid #e9ecef;
    padding-bottom: 12px;
  }

  .wiki-container h2 {
    font-size: 1.875em;
    font-weight: 600;
    line-height: 1.3;
    margin: 1.4em 0 8px 0;
    color: #37352f;
  }

  .wiki-container h3 {
    font-size: 1.5em;
    font-weight: 600;
    line-height: 1.3;
    margin: 1em 0 8px 0;
    color: #37352f;
  }

  /* Paragraphs */
  .wiki-container p {
    margin: 0 0 12px 0;
    line-height: 1.6;
  }

  /* Links */
  .wiki-container a {
    color: #0066cc;
    text-decoration: none;
    border-bottom: 1px solid rgba(0, 102, 204, 0.3);
    transition: border-color 0.2s ease;
  }

  .wiki-container a:hover {
    border-bottom-color: #0066cc;
  }

  /* Lists */
  .wiki-container ul, 
  .wiki-container ol {
    margin: 0 0 12px 0;
    padding-left: 1.5em;
  }

  .wiki-container li {
    margin: 4px 0;
    line-height: 1.6;
  }

  /* Code */
  .wiki-container code {
    background: rgba(135, 131, 120, 0.15);
    color: #eb5757;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-size: 0.9em;
    font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  }

  .wiki-container pre {
    background: #f7f6f3;
    padding: 16px;
    border-radius: 3px;
    overflow-x: auto;
    margin: 0 0 12px 0;
  }

  .wiki-container pre code {
    background: none;
    color: #37352f;
    padding: 0;
  }

  /* Mobile responsiveness */
  @media (max-width: 768px) {
    .wiki-container {
      padding: 24px;
    }
  }
</style>"#;

const NAVBAR: &str = r#"
<nav class="navbar bg-base-100 border-b border-base-300 px-6 sticky top-0 z-50 shadow-sm">
  <div class="flex-1">
      <a href="/" class="btn btn-ghost text-xl font-semibold">📚 Personal Wiki</a>
  </div>
  <div class="flex-none">
      <ul class="menu menu-horizontal px-1 gap-2">
          <li><a href="/" class="btn btn-ghost btn-sm">Home</a></li>
          <li><a href="https://github.com/AstraBert/personal-wiki" target="_blank" class="btn btn-ghost btn-sm">GitHub</a></li>
      </ul>
  </div>
</nav>
<br>
"#;

fn style_html(html: &str, username: &str) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n<title>{}'s Wiki</title>\n<script src=\"https://cdn.jsdelivr.net/npm/@tailwindcss/browser@4\"></script>\n<link href=\"https://cdn.jsdelivr.net/npm/daisyui@5/dist/full.css\" rel=\"stylesheet\" type=\"text/css\" />\n{}\n</head>\n<body>\n{}\n<div class=\"flex flex-col px-6 py-12 items-center justify-center wiki-container\">\n{}\n</div>\n</body>\n</html>",
        username, CSS_STYLE, NAVBAR, html
    )
}

fn hash_pwd(password: &str) -> Result<String, bcrypt::BcryptError> {
  hash(password, DEFAULT_COST)
}

fn verify_hashed_pwd(password: &str, hashed_password: &str) -> Result<bool, bcrypt::BcryptError> {
  verify(password, hashed_password)
}

struct Wiki {
  content: String,
  password: String,
}

impl Wiki {
  fn new(content: String, password: String) -> Self {
    Self {
      content: content,
      password: password,
    }
  }
}

async fn create_table() {
    let url = std::env::var("LIBSQL_CONNECTION_STRING").expect("LIBSQL_CONNECTION_STRING should be set");
    let token = std::env::var("LIBSQL_AUTH_TOKEN").expect("LIBSQL_AUTH_TOKEN should be set");
    let db = Builder::new_remote(url, token).build().await.expect("It should be possible to connect to remote database");
    let conn = db.connect().expect("It should be possible to connect to a local database");

    // Create a table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS wikis (id INTEGER PRIMARY KEY, user TEXT, content TEXT, password TEXT)",
        ()
    ).await.expect("It should be possible to create a table within the database");
}

async fn get_record(username: &str) -> Option<Wiki> {
    let url = std::env::var("LIBSQL_CONNECTION_STRING").expect("LIBSQL_CONNECTION_STRING should be set");
    let token = std::env::var("LIBSQL_AUTH_TOKEN").expect("LIBSQL_AUTH_TOKEN should be set");
    let db = Builder::new_remote(url, token).build().await.ok()?;
    let conn = db.connect().ok()?;

    let mut rows = conn.query("SELECT content, password FROM wikis WHERE user = ?", params![username]).await.ok()?;

    if let Some(row) = rows.next().await.ok()? {
        let content: String = row.get(0).ok()?;
        let pwd: String = row.get(1).ok()?;
        return Some(Wiki::new(content, pwd));
    }
    
    None
}
async fn insert_record(markdown_text: &str, username: &str, password: &str) -> Option<String> {
    create_table().await;
    let url = std::env::var("LIBSQL_CONNECTION_STRING").expect("LIBSQL_CONNECTION_STRING should be set");
    let token = std::env::var("LIBSQL_AUTH_TOKEN").expect("LIBSQL_AUTH_TOKEN should be set");
    let db = Builder::new_remote(url, token).build().await.ok()?;
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
                    "INSERT INTO wikis (user, content, password) VALUES (?1, ?2, ?3)", 
                    [username, &html_text, password]
                ).await.ok()?;
                return None
            }
        }
    }
    Some("Could not convert markdown text to HTML".to_string())
}
async fn update_record(markdown_text: &str, username: &str, password: &str) -> Option<String> {
    create_table().await;
    let url = std::env::var("LIBSQL_CONNECTION_STRING").expect("LIBSQL_CONNECTION_STRING should be set");
    let token = std::env::var("LIBSQL_AUTH_TOKEN").expect("LIBSQL_AUTH_TOKEN should be set");
    let db = Builder::new_remote(url, token).build().await.ok()?;
    let conn = db.connect().ok()?;
    let html_text = to_html(markdown_text);
    if html_text != markdown_text { // conversion happened correctly
        let user_exists = get_record(username).await;
        match user_exists {
            Some(r) => {
                let verification = verify_hashed_pwd(password, &r.password);
                match verification {
                    Ok(pwd_match) => {
                      if pwd_match {
                        conn.execute(
                            "UPDATE wikis SET content = ?1 WHERE user = ?2", 
                            [&html_text, username]
                        ).await.ok()?;
                        return None
                      } else {
                        return Some("Wrong username or password".to_string());
                      }
                    } 
                    Err(e) => {
                      return Some(e.to_string());
                    }
                }
            }
            None => {
                return Some("User does not exists".to_string())
            }
        }
    }
    Some("Could not convert markdown text to HTML".to_string())
}

async fn delete_record(username: &str, password: &str) -> Option<String> {
  create_table().await;
  let url = std::env::var("LIBSQL_CONNECTION_STRING").expect("LIBSQL_CONNECTION_STRING should be set");
  let token = std::env::var("LIBSQL_AUTH_TOKEN").expect("LIBSQL_AUTH_TOKEN should be set");
  let db = Builder::new_remote(url, token).build().await.ok()?;
  let conn = db.connect().ok()?;
  let user_exists = get_record(username).await;
  match user_exists {
    Some(r) => {
      let verification = verify_hashed_pwd(password, &r.password);
      match verification {
        Ok(pwd_match) => {
          if pwd_match {
            conn.execute("DELETE FROM wikis WHERE user = ?", params![username]).await.ok()?;
          } else {
            return Some("Wrong username or password".to_string())
          }
        }
        Err(e) => {
          return Some(e.to_string())
        }
      }
    }
    None => {
      return Some("User does not exist".to_string())
    }
  }
  None
}

#[derive(Deserialize, Derivative)]
#[derivative(Debug)]
struct CreateOrUpdateWikiRequest {
    content: String,
    username: String,
    #[derivative(Debug = "ignore")]
    password: String,
}

#[derive(Serialize, Debug)]
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

#[derive(Deserialize, Derivative)]
#[derivative(Debug)]
struct DeleteWikiRequest {
  username: String,
  #[derivative(Debug = "ignore")]
  password: String,
}

#[derive(Serialize, Debug)]
struct DeleteWikiResponse {
  success: bool,
  error: Option<String>,
}

#[instrument]
async fn create_wiki(Json(payload): Json<CreateOrUpdateWikiRequest>) -> Json<CreateOrUpdateWikiResponse> {
    let hashed_psw = hash_pwd(&payload.password);
    let password: String;
    match hashed_psw {
        Ok(s) => {
          password = s;
        }
        Err(e) => {
          error!(event = "CreateWiki", data_id = %payload.username, "{}", e.to_string());
          return Json(CreateOrUpdateWikiResponse::new(false, Some(e.to_string()), None))
        }
    }
    if let Some(error_msg) = insert_record(&payload.content, &payload.username, &password).await {
        error!(event = "CreateWiki", data_id = %payload.username, "{}", error_msg);
        return Json(CreateOrUpdateWikiResponse::new(false, Some(error_msg), None))
    }
    info!(event = "CreateWiki", data_id = %payload.username, "Wiki successfully created");
    Json(CreateOrUpdateWikiResponse::new(true, None, Some(format!("/wikis/{}", &payload.username))))
}

#[instrument]
async fn update_wiki(Json(payload): Json<CreateOrUpdateWikiRequest>) -> Json<CreateOrUpdateWikiResponse> {
    if let Some(error_msg) = update_record(&payload.content, &payload.username, &payload.password).await {
        error!(event = "UpdateWiki", data_id = %payload.username, "{}", error_msg);
        return Json(CreateOrUpdateWikiResponse::new(false, Some(error_msg), None))
    }
    info!(event = "UpdateWiki", data_id = %payload.username, "Wiki successfully updated");
    Json(CreateOrUpdateWikiResponse::new(true, None, Some(format!("/wikis/{}", &payload.username))))
}

#[instrument]
async fn get_wiki(Path(username): Path<String>) -> Html<String> {
    match get_record(&username).await {
        Some(content) => {
            let styled_content = style_html(&content.content, &username);
            info!(event = "GetWiki", data_id = %username, "Wiki successfully retrieved");
            return Html(styled_content)
        }
        ,
        None => {
          error!(event = "GetWiki", data_id = %username, "Wiki not found for user {}", username);
          return Html(format!("Wiki for user {} not found... Please create one and try again!", &username))
        }
    }
}

#[instrument]
async fn delete_wiki(Json(payload): Json<DeleteWikiRequest>) -> Json<DeleteWikiResponse> {
  match delete_record(&payload.username, &payload.password).await {
      Some(s) => {
        error!(event = "DeleteWiki", data_id = %payload.username, "{}", s);
        return Json(DeleteWikiResponse { success: false, error: Some(s) })
      }
      None => {
        info!(event = "DeleteWiki", data_id = %payload.username, "Wiki successfully deleted");
        return Json(DeleteWikiResponse { success: true, error: None })
      }
  }
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt().pretty().init();
  let index_html = ServeFile::new("./pages/index.html");
  let scripts = ServeDir::new("./scripts/");
  let app = Router::new()
    .route("/wikis", post(create_wiki).patch(update_wiki).delete(delete_wiki))
    .route("/wikis/{username}", get(get_wiki))
    .nest_service("/scripts", scripts)
    .route_service("/", index_html);
  let address = "0.0.0.0:3000";
  let listener = tokio::net::TcpListener::bind(address).await.unwrap();
  println!("Starting to serving application on {}", address);
  axum::serve(listener, app).await.unwrap();
}
