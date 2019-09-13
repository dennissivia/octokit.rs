// use chrono::{DateTime, NaiveDateTime, Utc};
use openssl;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, LINK, USER_AGENT};
use reqwest::Response;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{encode, Algorithm, Header};

pub mod apps;
pub mod error;
pub mod webhooks;
use error::OctokitError;

type ID = u64;
type CommitSha = String;

// TODO replace with proper URI type later
type URI = String;

// TODO replace with chrone::DateTime
type DateTime = String;

/// Global Relay ID for GQL queries with Node ID
type GRID = String;

type NameWithOwner = String;

#[derive(Serialize, Deserialize, Debug)]
pub enum OwnerType {
    User,
    Bot,
    Organization,
}

type Email = String;

#[derive(Serialize, Deserialize, Debug)]
enum GithubEvent {
    Integration,
    Installation,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum GithubEventAction {
    Created,
    Removed,
    Deleted,
}

enum ApiPreviews {
    Antiope,
    MachineMan,
}

impl fmt::Display for ApiPreviews {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiPreviews::Antiope => write!(f, "{}", "application/vnd.github.antiope-preview+json"),
            ApiPreviews::MachineMan => {
                write!(f, "{}", "application/vnd.github.machine-man-preview+json")
            }
        }
    }
}

impl ApiPreviews {
    fn to_media_type(&self) -> String {
        self.to_string()
    }
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
    r#type: OwnerType,
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
#[derive(Deserialize, Debug)]
struct CommentPayload {
    pub action: GithubEventAction,
    pub issue: Issue,
    pub repository: Repository,
    pub comment: IssueComment,
}

#[derive(Deserialize, Debug)]
pub struct CommitAuthor {
    pub name: String,
    pub email: Email,
}

/// Used for web-flows etc, that are on behalf of a given user
type Committer = CommitAuthor;

#[derive(Deserialize, Debug)]
pub struct Commit {
    pub id: CommitSha,
    pub tree_id: CommitSha,
    pub distinct: bool,
    pub message: String,
    pub author: CommitAuthor,
    pub committer: Committer,
    pub url: URI,
}

#[derive(Deserialize, Debug)]
pub struct Pusher {
    pub name: String,
    pub email: Email,
}

#[derive(Deserialize, Debug)]
pub struct PushInstallation {
    pub id: ID,
    pub node_id: GRID,
}

#[derive(Deserialize, Debug)]
pub struct PushPayload {
    pub r#ref: String,
    pub before: CommitSha,
    pub after: CommitSha,
    pub created: bool,
    pub deleted: bool,
    pub forced: bool,
    pub base_ref: Option<String>,
    pub compare: URI,
    pub commits: Vec<Commit>,
    pub head_commit: Option<Commit>,
    pub repository: Repository,
    pub pusher: Pusher,
    pub sender: User,
    pub installation: Option<PushInstallation>,
}

#[derive(Deserialize, Debug)]
struct IssuePayload {
    pub issue: Issue,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PermissionGrant {
    Read,  // GH read-only permission
    Write, // GH read-write permission
}

// TODO consider making this a hashmap if no-access is implemented by omitting the key
#[derive(Serialize, Deserialize, Debug)]
pub struct InstallationPermissions {
    pub administration: Option<PermissionGrant>,
    pub packages: Option<PermissionGrant>,
    pub statuses: Option<PermissionGrant>,
    pub issues: Option<PermissionGrant>,
    pub deployments: Option<PermissionGrant>,
    pub contents: Option<PermissionGrant>,
    pub checks: Option<PermissionGrant>,
    pub vulnerability_alerts: Option<PermissionGrant>,
    pub pull_requests: Option<PermissionGrant>,
    pub pages: Option<PermissionGrant>,
    pub metadata: Option<PermissionGrant>,
    pub app_config: Option<PermissionGrant>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PullRequest {
    pub id: ID,
    pub url: URI,
    pub number: u64,
    pub state: String,
    pub locked: bool,
    pub title: String,
    pub body: String,
    pub user: User,
    pub node_id: GRID,
    pub html_url: URI,
    pub diff_url: URI,
    pub patch_url: URI,
    pub issue_url: URI,
    pub commits_url: URI,
    pub review_comments_url: URI,
    pub review_comment_url: URI,
    pub comments_url: URI,
    pub statuses_url: URI,
}

#[derive(Deserialize, Debug)]
pub struct GithubApp {
    pub id: ID,
    pub slug: String,
    pub node_id: GRID,
    pub owner: User,
    pub name: String,
    pub description: String,
    pub external_url: URI,
    pub html_url: URI,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    // TODO should we just use a HashMap to supports it's dynamic nature?
    //    pub permissions: HashMap<String, PermissionGrant>,
    pub permissions: InstallationPermissions,
    pub evens: Vec<String>,
    pub installations_count: Option<u64>, // only included in authenticated calls
}

// TODO create CheckSuite status stuff
#[derive(Deserialize, Debug)]
pub struct CheckSuite {
    pub id: ID,
    pub node_id: GRID,
    pub head_branch: String,
    pub head_sha: CommitSha,
    pub status: String,
    pub conclusion: String,
    pub url: URI,
    pub before: CommitSha,
    pub after: CommitSha,
    pub pull_requests: Vec<PullRequest>,
    pub app: GithubApp,
    pub repository: Repository,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallationRepository {
    pub id: ID,
    pub node_id: ID,
    pub name: String,
    pub full_name: String,
    pub private: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Installation {
    pub id: ID,
    //pub account: User, // OWNER?
    pub repository_selection: String,
    pub access_tokens_url: URI,
    pub repositories_url: URI,
    pub html_url: URI,
    pub app_id: ID,
    pub target_id: ID,
    pub target_type: OwnerType,
    pub permissions: InstallationPermissions,
    pub events: Vec<String>,
    //    pub created_at: DateTime, // Ignore! because they are delivered as timestamps
    //    pub updated_at: DateTime, // Ignore! because they are delivered as timestamps
    pub single_file_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct InstallationPayload {
    // FIXME create event-action enum
    pub action: GithubEventAction,
    pub installation: Installation,
    // in case of deleted installs there is no repository key
    pub repositories: Option<Vec<InstallationRepository>>,
    pub sender: User,
}

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

#[derive(Deserialize, Debug)]
pub struct InstallationToken {
    pub token: String,
    pub expires_at: DateTime,
    pub permissions: InstallationPermissions,
    // omitted if repository_ids is not set in the request
    pub repositories: Option<Vec<Repository>>,
}

fn perform_get(
    token: &String,
    url: URI,
    token_type: AuthTokenType,
) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();

    client
        .get(&url[..])
        .header(USER_AGENT, "Octokit/Rust v0.1.0")
        .header(ACCEPT, "application/vnd.github.antiope-preview+json")
        .header(
            AUTHORIZATION,
            String::from(format!("{} {}", token_type, token)),
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
    token_type: AuthTokenType,
    media_type: String,
) -> Result<Response, reqwest::Error> {
    println!(
        "media type is: {}, token_type is: {}",
        media_type, token_type
    );
    let client = reqwest::Client::new();
    let result = client
        .post(&url[..])
        .header(USER_AGENT, "Octokit/Rust v0.1.0")
        .header(ACCEPT, media_type)
        .header(
            AUTHORIZATION,
            String::from(format!("{} {}", token_type, token)),
        )
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

    let res = perform_post(
        &token,
        url,
        &new_comment,
        AuthTokenType::Token,
        ApiPreviews::Antiope.to_media_type(),
    );
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
    // TODO replace naive conversion with proper error handling
    let result = perform_get(&token, url, AuthTokenType::Token);
    match result {
        Ok(mut response) => {
            println!("Request succeeded :D {:?}", response);
            let data: Vec<ReviewComment> = response.json().expect("decoding comments failed");
            println!("Data: {:?}", data);
            Some(data)
        }
        Err(err) => {
            println!("Request failed {:?}", err);
            None
        }
    }
}

pub fn get_issue_batch(_token: &String) {
    unimplemented!()
}
pub fn get_pull_requests(_token: &String, _nwo: &String) {
    unimplemented!()
}

// Status: TODO
//TODO paginate over all issues later
pub fn get_all_issues(token: &String) {
    get_issue_batch(&token);
}

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
            let data: Vec<ReviewComment> = response.json().expect("decoding comments failed");
            println!("Data: {:?}", data);
            Some(data)
        }
        Err(err) => {
            println!("Request failed {:?}", err);
            None
        }
    }
}

type GithubAppId = String;

/// Well-known JWT claims
///    iss (issuer): Issuer of the JWT
///    exp (expiration time): Time after which the JWT expires
///    iat (issued at time): Time at which the JWT was issued; can be used to determine age of the JWT
///    sub (subject): Subject of the JWT (the user)
///    aud (audience): Recipient for which the JWT is intended
///    nbf (not before time): Time before which the JWT must not be accepted for processing
///    jti (JWT ID): Unique identifier; can be used to prevent the JWT from being replayed (allows a token to be used only once)
// Implement the minimum required by Github (for now)
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iat: u64,
    exp: u64,
    iss: GithubAppId,
}

enum AuthTokenType {
    Token,
    JWT,
}

impl fmt::Display for AuthTokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthTokenType::Token => write!(f, "{}", "token"),
            AuthTokenType::JWT => write!(f, "{}", "Bearer"),
        }
    }
}

/// See: https://developer.github.com/apps/building-github-apps/authenticating-with-github-apps/#authenticating-as-a-github-app
/// Github expects RS256 encoded JWTs
/// ISS has to be the APP_ID
/// Private key is in PKCS#1 RSAPrivateKey format
/// https://developer.github.com/apps/building-github-apps/authenticating-with-github-apps/#generating-a-private-key
pub fn create_jwt(path: &str, app_id: &String) -> std::result::Result<String, String> {
    println!("opening secret key file: {}", path);
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

///  curl -i -H "Authorization: Bearer YOUR_JWT"
///          -H "Accept: application/vnd.github.machine-man-preview+json"
///          https://api.github.com/app
pub fn get_app(token: &String) -> Option<GithubApp> {
    let result = perform_get(
        &token,
        "https://api.github.com/app".to_string(),
        AuthTokenType::JWT,
    );
    //    println!("result is: {:?}", result);
    match result {
        Ok(mut response) => {
            if response.status() == reqwest::StatusCode::OK {
                let app: GithubApp = response.json().unwrap();
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
            println!("request failed. {:?}", error);
            return None;
        }
    }
}

/// GET /app/installations/:installation_id
pub fn get_installation() -> () {}

#[derive(Serialize, Debug)]
struct CreateInstallationToken {
    #[serde(skip_serializing_if = "Option::is_none")]
    repository_ids: Option<Vec<ID>>,
    permissions: HashMap<String, PermissionGrant>,
}

#[derive(Serialize, Debug)]
struct CreateCheckSuite {
    head_sha: CommitSha,
}

///  POST /repos/:owner/:repo/check-suites
pub fn create_check_suite(
    token: &String,
    nwo: &NameWithOwner,
    sha: CommitSha,
) -> Option<CheckSuite> {
    let data = CreateCheckSuite {
        head_sha: sha.to_string(),
    };

    let result = perform_post(
        &token,
        format!("https://api.github.com/repos/{}/check-suites", nwo),
        &data,
        AuthTokenType::Token,
        ApiPreviews::Antiope.to_media_type(),
    );

    println!("result is: {:?}", result);
    match result {
        Ok(mut response) => {
            if response.status() == reqwest::StatusCode::OK {
                let check_suite: CheckSuite = response.json().unwrap();
                return Some(check_suite);
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
            println!("request failed. {:?}", error);
            return None;
        }
    }
}

#[derive(Serialize, Debug)]
pub struct CreateCheckRun {
    pub name: String,
    pub head_sha: CommitSha,
    //    status: Option<String>,
    //    conclusion: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CheckRun {
    pub id: ID,
    pub head_sha: CommitSha,
    pub node_id: GRID,
    pub external_id: String,
    pub url: URI,
    pub html_url: URI,
    pub details_url: URI,
    pub status: String,
    pub conclusion: Option<String>,
    pub started_at: DateTime,
    pub completed_at: Option<DateTime>,
}

///  POST /repos/:owner/:repo/check-runs
pub fn create_check_run(token: &String, nwo: &NameWithOwner, sha: CommitSha) -> Option<CheckRun> {
    let data = CreateCheckRun {
        name: String::from("Example Check-Run"),
        head_sha: sha.to_string(),
    };

    let result = perform_post(
        &token,
        format!("https://api.github.com/repos/{}/check-runs", nwo),
        &data,
        AuthTokenType::Token,
        ApiPreviews::Antiope.to_media_type(),
    );

    println!("result is: {:?}", result);
    match result {
        Ok(mut response) => {
            if response.status() == reqwest::StatusCode::CREATED {
                let check_run: CheckRun = response.json().unwrap();
                return Some(check_run);
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
            println!("Request failed. {:?}", error);
            return None;
        }
    }
}

/// POST /app/installations/:installation_id/access_tokens
pub fn create_installation_token(jwt: String, installation_id: ID) -> Option<String> {
    let mut permissions = HashMap::new();
    permissions.insert(String::from("checks"), PermissionGrant::Write);

    // only allow a certain list of repositories. Not all
    //    let repository_ids = Some(vec![]);
    let repository_ids = None;

    let data = CreateInstallationToken {
        repository_ids,
        permissions,
    };

    let result = perform_post(
        &jwt,
        format!(
            "https://api.github.com/app/installations/{}/access_tokens",
            installation_id
        ),
        &data,
        AuthTokenType::JWT,
        ApiPreviews::MachineMan.to_media_type(),
    );

    println!("result is: {:?}", result);
    match result {
        Ok(mut response) => {
            // We expect a 201
            if response.status() == reqwest::StatusCode::CREATED {
                let token_data: InstallationToken = response.json().unwrap();
                println!("{:?}", token_data);
                return Some(token_data.token);
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
            println!("Request failed. {:?}", error);
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
