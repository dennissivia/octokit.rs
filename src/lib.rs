// use chrono::{DateTime, NaiveDateTime, Utc};
use openssl;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, LINK, USER_AGENT};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

extern crate jsonwebtoken as jwt;
use jwt::{decode, encode, Algorithm, Header, Validation};

// use serde_json::{Error, Result};

type ID = u64;
type CommitSha = String;
// TODO replace with proper URI type later
type URI = String;
// TODO replace with chrone::DateTime
type DateTime = String;
// Global Relay ID for GQL queries with Node ID
type GRID = String;
type NameWithOwner = String;
type AuthToken = String;

enum OwnerType {
    User,
    Bot,
    Organization,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiError {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: ID,
    pub login: String,
    pub node_id: GRID,
    // type: OwnerType,
    site_admin: bool,
    // avatar_url: URI,
    // TODO consider a hypermedia mixin or something
    // gravatar_id: String,
    // url: URI,
    // html_url: URI,
    // followers_url: URI,
    // following_url: URI,
    // gists_url: URI,
    // starred_url: URI,
    // subscriptions_url: URI,
    // organizations_url: URI,
    // repos_url: URI,
    // events_url: URI,
    // received_events_url: URI
}

/// https://developer.github.com/v3/issues/comments/#response-3
#[derive(Serialize, Deserialize, Debug)]
pub struct IssueComment {
    pub id: ID,
    pub body: String,
    pub user: User,
    // url: URI,
    // html_url: URI,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Issue {
    pub id: ID,
    pub number: ID,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    pub full_name: String,
}

// mod webhook_payloads {
#[derive(Serialize, Deserialize, Debug)]
struct CommentPayload {
    pub action: String,
    pub issue: Issue,
    pub repository: Repository,
    pub comment: IssueComment,
}

#[derive(Serialize, Deserialize, Debug)]
struct IssuePayload {
    pub issue: Issue,
}
// }

// TODO create Review and ReviewComment payloads
// struct Review
#[derive(Serialize, Deserialize, Debug)]
pub struct Review {}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReviewComment {
    pub id: ID,
    pub body: String,
    pub user: User,
    // node_id: String,
    // pull_request_review_id: ID,
    // diff_hunk: String,
    // path: String,
    // position: u32,
    // original_position: u32,
    // commit_id: CommitSha,
    // original_commit_id: CommitSha,
    // in_reply_to_id: ID,
    // author_association: String
    // created_at: DateTime,
    // updated_at: DateTime,
    // html_url: URI,
    // pull_request_url: URI,
    // url: URI,
}

// payloads for Create and Update
#[derive(Serialize, Debug)]
struct CreateComment {
    body: String,
}

#[derive(Debug)]
pub struct OctokitError {
    details: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct App {
    id: ID,
    installations_count: usize,
    node_id: GRID,
    name: String,
    description: String,
    external_url: URI,
    html_url: URI,
    created_at: DateTime,
    updated_at: DateTime,
    //owner: Owner
}

impl OctokitError {
    fn new(msg: &str) -> OctokitError {
        OctokitError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for OctokitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for OctokitError {
    fn description(&self) -> &str {
        &self.details
    }
}

impl From<serde_json::Error> for OctokitError {
    fn from(err: serde_json::Error) -> Self {
        OctokitError::new(err.description())
    }
}

impl From<reqwest::Error> for OctokitError {
    fn from(err: reqwest::Error) -> Self {
        OctokitError::new(err.description())
    }
}

fn perform_get(
    token: &String,
    url: URI,
    token_type: AuthTokenType,
) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let token_prefix = match token_type {
        AuthTokenType::Token => "token",
        AuthTokenType::Bearer => "Bearer",
    };

    client
        .get(&url[..])
        .header(USER_AGENT, "Octokit/Rust v0.1.0")
        .header(CONTENT_TYPE, "application/vnd.github.antiope-preview+json")
        .header(
            AUTHORIZATION,
            String::from(format!("{} {}", token_prefix, token)),
        )
        .send()
}

fn perform_delete(token: &String, url: URI) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    client
        .delete(&url[..])
        .header(USER_AGENT, "Octokit/Rust v0.1.0")
        .header(CONTENT_TYPE, "application/vnd.github.antiope-preview+json")
        .header(AUTHORIZATION, String::from(format!("token {}", token)))
        .send()
}

// TODO consider using custom types instead of reqwest types
fn perform_post<T: Serialize>(
    token: &String,
    url: URI,
    data: &T,
) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let result = client
        .post(&url[..])
        .header(USER_AGENT, "Octokit/Rust v0.1.0")
        .header(CONTENT_TYPE, "application/vnd.github.antiope-preview+json")
        .header(AUTHORIZATION, String::from(format!("token {}", token)))
        .json(&data)
        .send();

    // just for debugging
    result
}

/// DELETE /repos/:owner/:repo/issues/comments/:comment_id
pub fn delete_issue_comment(token: &String, nwo: &NameWithOwner, comment_number: ID) {
    let url: String = format!(
        "https://api.github.com/repos/{}/issues/comments/{}",
        nwo, comment_number
    );
    let result = perform_delete(token, url);
    match result {
        Ok(_) => {
            println! {"DELETE succeeded"};
        }
        Err(err) => {
            println!("DELETE failed {:}", err);
        }
    }
}

pub fn create_issue_comment(
    token: &String,
    issue_number: ID,
    repo_name: &String,
    message: String,
) -> Result<IssueComment, OctokitError> {
    let new_comment = CreateComment { body: message };
    let url: String = format!(
        "https://api.github.com/repos/{}/issues/{}/comments",
        repo_name, issue_number
    );

    let res = perform_post(&token, url, &new_comment);
    match res {
        Ok(mut response) => {
            println!("Request succeeded {:?}", response);
            let created_comment: IssueComment = response.json().expect("JSON parse failed");
            println!("{:?}", created_comment);
            return Ok(created_comment);
        }
        Err(err) => {
            println!("Request failed {:?}", err);
            return Err(OctokitError::from(err));
        }
    }
}

// Status: TODO
/// GET /repos/:owner/:repo/pulls/:pull_number/comments
pub fn get_review_comments(
    token: &String,
    nwo: NameWithOwner,
    pull_number: ID,
) -> Option<Vec<ReviewComment>> {
    let url: String = format!(
        "https://api.github.com/repos/{}/pulls/{}/comments",
        nwo, pull_number
    );
    perform_get(&token, url, AuthTokenType::Token);
    None
}

// Status: TODO
fn get_issue_batch(token: &String) {}

// Status: TODO
pub fn get_all_issues(token: &String) {
    //TODO paginate over all issues later
    get_issue_batch(&token);
}

// Status: TODO
fn get_pull_requests(token: &String, nwo: &String) {}

// Status: WIP (partially done)
// GET /repos/:owner/:repo/pulls/comments
pub fn get_all_review_comments(token: &String, nwo: &NameWithOwner) -> Option<Vec<ReviewComment>> {
    let url: String = format!("https://api.github.com/repos/{}/comments", nwo);
    let result = perform_get(&token, url, AuthTokenType::Token);

    match result {
        Ok(mut response) => {
            println!("Request succeeded :D {:?}", response);
            let link_header = response.headers().get(LINK);
            if link_header.is_some() {
                println!("link header: {:?}", link_header);
            }
            let data: Vec<ReviewComment> = response.json().expect("die");
            println!("Data: {:?}", data);
            Some(data)
        }
        Err(err) => {
            println!("Request failed {:?}", err);
            None
        }
    }
}

type GitHubAppId = String;

/// Well-known JWT claims
///    iss (issuer): Issuer of the JWT
///    exp (expiration time): Time after which the JWT expires
///    iat (issued at time): Time at which the JWT was issued; can be used to determine age of the JWT
///    sub (subject): Subject of the JWT (the user)
///    aud (audience): Recipient for which the JWT is intended
///    nbf (not before time): Time before which the JWT must not be accepted for processing
///    jti (JWT ID): Unique identifier; can be used to prevent the JWT from being replayed (allows a token to be used only once)
// Implement the minimum required by GitHub (for now)
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iat: u64,
    exp: u64,
    iss: GitHubAppId,
}

enum AuthTokenType {
    Token,
    Bearer,
}

/// See: https://developer.github.com/apps/building-github-apps/authenticating-with-github-apps/#authenticating-as-a-github-app
/// GitHub expects RS256 encoded JWTs
/// ISS has to be the APP_ID
/// Private key is in PKCS#1 RSAPrivateKey format
/// https://developer.github.com/apps/building-github-apps/authenticating-with-github-apps/#generating-a-private-key
pub fn generate_jwt(path: &str, app_id: &String) -> std::result::Result<String, String> {
    println!("opening event payload file: {}", path);
    let mut file = File::open(path).map_err(|_| "failed to open file".to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|_| "Failed t read from file".to_string())?;

    let now = SystemTime::now();
    let iat = now
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let exp = iat + 60 * 10; // 10 minute validity
    let iss = app_id;

    //    openssl.private_key_from_pem
    let key = openssl::rsa::Rsa::private_key_from_pem(contents.as_bytes())
        .map_err(|_| "Openssl died".to_string())?;
    let der = key
        .private_key_to_der()
        .map_err(|_| "creating der failed")?;

    let claims = Claims {
        iat: iat,
        exp: exp,
        iss: iss.to_string(),
    };
    let mut header = Header::default();
    header.alg = Algorithm::RS256;
    let token = encode(&header, &claims, der.as_ref()).map_err(|_| "JWT encoding failed")?;

    return Ok(token);
}

pub fn get_app(token: &String) -> Option<App> {
    //    curl -i -H "Authorization: Bearer YOUR_JWT" -H "Accept: application/vnd.github.machine-man-preview+json" https://api.github.com/app
    let result = perform_get(
        &token,
        "https://api.github.com/app".to_string(),
        AuthTokenType::Bearer,
    );
    //    println!("result is: {:?}", result);
    match result {
        Ok(mut response) => {
            if response.status() == reqwest::StatusCode::OK {
                let app: App = response.json().unwrap();
                return Some(app);
            } else if response.status() == reqwest::StatusCode::UNAUTHORIZED {
                let body: ApiError = response.json().expect("Its all broken");
                println!("{:?}", body);
                return None;
            } else {
                let status = response.headers().get("status").unwrap();
                println!("Request failed with status: {:?}", status);
                return None;
            }
        }
        Err(error) => {
            println!("request failed. How do we get the error payload?");
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    // Write tests for JWT logic (no API mocks needed)
    // Write tests for error cases for non-API functions
}
