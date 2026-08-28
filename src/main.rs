use axum::{
    extract::{DefaultBodyLimit, Path, Query, Request, State},
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{Duration, Utc};
use futures::Stream;
use hmac::{Hmac, Mac};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    FromRow, SqlitePool,
};
use std::{
    collections::HashMap,
    convert::Infallible,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};
use tokio::sync::{broadcast, Mutex};
use tokio_stream::{wrappers::BroadcastStream, StreamExt};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use url::Url;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;
const FREE_CUE_LIMIT: usize = 12;

#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    channels: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    cue_lock: Arc<Mutex<()>>,
    public_url: String,
}

#[derive(Debug, thiserror::Error)]
enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("Room or access token not found")]
    NotFound,
    #[error("This controller is still waiting for host approval")]
    Pending,
    #[error("This room has expired")]
    Expired,
    #[error("Internal server error")]
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Pending => StatusCode::FORBIDDEN,
            Self::Expired => StatusCode::GONE,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(serde_json::json!({"error": self.to_string()}))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(error = %err, "database error");
        Self::Internal
    }
}

#[derive(Deserialize)]
struct CueInput {
    scene: String,
    name: String,
}

#[derive(Deserialize)]
struct CreateRoom {
    title: String,
    cues: Vec<CueInput>,
    webhook_url: Option<String>,
    webhook_secret: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct CreateRoomResponse {
    code: String,
    host_token: String,
    join_url: String,
    expires_at: String,
}

#[derive(FromRow, Clone)]
struct RoomRow {
    id: String,
    code: String,
    title: String,
    host_token_hash: String,
    join_secret_hash: String,
    current_index: i64,
    webhook_url: Option<String>,
    webhook_secret: Option<String>,
    expires_at: String,
}

#[derive(Serialize, FromRow, Clone)]
struct Cue {
    id: String,
    position: i64,
    scene: String,
    name: String,
}

#[derive(Serialize, FromRow)]
struct ControllerView {
    id: String,
    name: String,
    status: String,
    created_at: String,
}

#[derive(Serialize, FromRow, Clone)]
struct CueEvent {
    id: String,
    sequence: i64,
    cue_id: String,
    scene: String,
    cue_name: String,
    controller_name: String,
    webhook_status: String,
    created_at: String,
}

#[derive(Serialize)]
struct Snapshot {
    code: String,
    title: String,
    role: String,
    status: String,
    current_index: i64,
    cues: Vec<Cue>,
    controllers: Vec<ControllerView>,
    events: Vec<CueEvent>,
    expires_at: String,
    webhook_configured: bool,
}

#[derive(Deserialize)]
struct TokenQuery {
    token: String,
}

#[derive(Deserialize)]
struct JoinInput {
    secret: String,
    name: String,
}

#[derive(Serialize)]
struct JoinResponse {
    token: String,
    status: String,
}

#[derive(Deserialize)]
struct SetCueInput {
    index: i64,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    build_sha: String,
}

fn hash_token(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
fn random_token(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
fn valid_name(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.chars().count() <= max && !s.chars().any(char::is_control)
}

async fn room_by_code(db: &SqlitePool, code: &str) -> Result<RoomRow, ApiError> {
    let room = sqlx::query_as::<_, RoomRow>("SELECT id, code, title, host_token_hash, join_secret_hash, current_index, webhook_url, webhook_secret, expires_at FROM rooms WHERE code = ?")
        .bind(code.to_uppercase()).fetch_optional(db).await?.ok_or(ApiError::NotFound)?;
    if room
        .expires_at
        .parse::<chrono::DateTime<Utc>>()
        .map_err(|_| ApiError::Internal)?
        <= Utc::now()
    {
        return Err(ApiError::Expired);
    }
    Ok(room)
}

async fn role_for(
    db: &SqlitePool,
    room: &RoomRow,
    token: &str,
) -> Result<(String, String), ApiError> {
    let hashed = hash_token(token);
    if hashed == room.host_token_hash {
        return Ok(("host".into(), "approved".into()));
    }
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT name, status FROM controllers WHERE room_id = ? AND token_hash = ?")
            .bind(&room.id)
            .bind(hashed)
            .fetch_optional(db)
            .await?;
    row.map(|(_, status)| ("controller".into(), status))
        .ok_or(ApiError::NotFound)
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        // Docker supplies BUILD_SHA at compile time. `dev` keeps local builds
        // runnable while remaining an explicit, non-ambiguous identity.
        build_sha: option_env!("BUILD_SHA").unwrap_or("dev").to_string(),
    })
}

async fn create_room(
    State(state): State<AppState>,
    Json(input): Json<CreateRoom>,
) -> Result<(StatusCode, Json<CreateRoomResponse>), ApiError> {
    if !valid_name(&input.title, 80) {
        return Err(ApiError::BadRequest(
            "Show name must be 1–80 characters".into(),
        ));
    }
    if input.cues.is_empty() || input.cues.len() > FREE_CUE_LIMIT {
        return Err(ApiError::BadRequest(format!(
            "Add between 1 and {FREE_CUE_LIMIT} cues"
        )));
    }
    for cue in &input.cues {
        if !valid_name(&cue.scene, 60) || !valid_name(&cue.name, 100) {
            return Err(ApiError::BadRequest(
                "Each scene and cue needs a short name".into(),
            ));
        }
    }
    if let Some(ref raw_url) = input.webhook_url {
        validate_public_webhook(raw_url)?;
        validate_resolved_webhook(raw_url).await?;
    }
    if input.webhook_url.is_some() && input.webhook_secret.as_deref().unwrap_or("").len() < 12 {
        return Err(ApiError::BadRequest(
            "Webhook signing secret must be at least 12 characters".into(),
        ));
    }
    let id = Uuid::new_v4().to_string();
    let host_token = random_token(40);
    let join_secret = random_token(24);
    let expires_at = Utc::now() + Duration::hours(8);
    let now = Utc::now().to_rfc3339();
    let mut tx = state.db.begin().await?;
    let mut code = String::new();
    for _ in 0..8 {
        code = random_code();
        let inserted = sqlx::query("INSERT OR IGNORE INTO rooms (id, code, title, host_token_hash, join_secret_hash, webhook_url, webhook_secret, created_at, expires_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(&id).bind(&code).bind(input.title.trim()).bind(hash_token(&host_token)).bind(hash_token(&join_secret))
            .bind(input.webhook_url.as_deref()).bind(input.webhook_secret.as_deref()).bind(&now).bind(expires_at.to_rfc3339()).execute(&mut *tx).await?;
        if inserted.rows_affected() == 1 {
            break;
        }
    }
    if code.is_empty() {
        return Err(ApiError::Internal);
    }
    for (position, cue) in input.cues.iter().enumerate() {
        sqlx::query("INSERT INTO cues (id, room_id, position, scene, name) VALUES (?, ?, ?, ?, ?)")
            .bind(Uuid::new_v4().to_string())
            .bind(&id)
            .bind(position as i64)
            .bind(cue.scene.trim())
            .bind(cue.name.trim())
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    let join_url = format!(
        "{}/join/{}?secret={}",
        state.public_url.trim_end_matches('/'),
        code,
        join_secret
    );
    Ok((
        StatusCode::CREATED,
        Json(CreateRoomResponse {
            code,
            host_token,
            join_url,
            expires_at: expires_at.to_rfc3339(),
        }),
    ))
}

fn random_code() -> String {
    const CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| CHARS[rng.gen_range(0..CHARS.len())] as char)
        .collect()
}

fn validate_public_webhook(raw: &str) -> Result<(), ApiError> {
    let url =
        Url::parse(raw).map_err(|_| ApiError::BadRequest("Webhook URL is not valid".into()))?;
    if url.scheme() != "https" {
        return Err(ApiError::BadRequest("Webhook URL must use HTTPS".into()));
    }
    let host = url
        .host_str()
        .ok_or_else(|| ApiError::BadRequest("Webhook URL needs a public host".into()))?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err(ApiError::BadRequest(
            "Webhook URL cannot contain credentials".into(),
        ));
    }
    if host == "localhost" || host.ends_with(".local") {
        return Err(ApiError::BadRequest(
            "Webhook cannot target a local address".into(),
        ));
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        let private = match ip {
            IpAddr::V4(v) => {
                v.is_private()
                    || v.is_loopback()
                    || v.is_link_local()
                    || v.is_unspecified()
                    || v.is_multicast()
            }
            IpAddr::V6(v) => {
                v.is_loopback() || v.is_unspecified() || v.is_unique_local() || v.is_multicast()
            }
        };
        if private {
            return Err(ApiError::BadRequest(
                "Webhook cannot target a private address".into(),
            ));
        }
    }
    Ok(())
}

async fn validate_resolved_webhook(raw: &str) -> Result<(), ApiError> {
    let url =
        Url::parse(raw).map_err(|_| ApiError::BadRequest("Webhook URL is not valid".into()))?;
    let host = url
        .host_str()
        .ok_or_else(|| ApiError::BadRequest("Webhook URL needs a public host".into()))?;
    let addresses = tokio::net::lookup_host((host, url.port_or_known_default().unwrap_or(443)))
        .await
        .map_err(|_| ApiError::BadRequest("Webhook host could not be resolved".into()))?;
    for address in addresses {
        let private = match address.ip() {
            IpAddr::V4(v) => {
                v.is_private() || v.is_loopback() || v.is_link_local() || v.is_unspecified()
            }
            IpAddr::V6(v) => v.is_loopback() || v.is_unspecified() || v.is_unique_local(),
        };
        if private {
            return Err(ApiError::BadRequest(
                "Webhook cannot resolve to a private address".into(),
            ));
        }
    }
    Ok(())
}

async fn join_room(
    Path(code): Path<String>,
    State(state): State<AppState>,
    Json(input): Json<JoinInput>,
) -> Result<(StatusCode, Json<JoinResponse>), ApiError> {
    let room = room_by_code(&state.db, &code).await?;
    if hash_token(&input.secret) != room.join_secret_hash {
        return Err(ApiError::NotFound);
    }
    if !valid_name(&input.name, 40) {
        return Err(ApiError::BadRequest(
            "Your controller name must be 1–40 characters".into(),
        ));
    }
    let token = random_token(40);
    sqlx::query("INSERT INTO controllers (id, room_id, name, token_hash, status, created_at) VALUES (?, ?, ?, ?, 'pending', ?)")
        .bind(Uuid::new_v4().to_string()).bind(&room.id).bind(input.name.trim()).bind(hash_token(&token)).bind(Utc::now().to_rfc3339()).execute(&state.db).await?;
    notify(&state, &room.code, "controller_pending").await;
    Ok((
        StatusCode::CREATED,
        Json(JoinResponse {
            token,
            status: "pending".into(),
        }),
    ))
}

async fn snapshot(
    Path(code): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<TokenQuery>,
) -> Result<Json<Snapshot>, ApiError> {
    let room = room_by_code(&state.db, &code).await?;
    let (role, status) = role_for(&state.db, &room, &query.token).await?;
    let cues = sqlx::query_as::<_, Cue>(
        "SELECT id, position, scene, name FROM cues WHERE room_id = ? ORDER BY position",
    )
    .bind(&room.id)
    .fetch_all(&state.db)
    .await?;
    let controllers = if role == "host" {
        sqlx::query_as::<_, ControllerView>("SELECT id, name, status, created_at FROM controllers WHERE room_id = ? ORDER BY created_at").bind(&room.id).fetch_all(&state.db).await?
    } else {
        Vec::new()
    };
    let events = sqlx::query_as::<_, CueEvent>("SELECT id, sequence, cue_id, scene, cue_name, controller_name, webhook_status, created_at FROM cue_events WHERE room_id = ? ORDER BY created_at DESC LIMIT 100").bind(&room.id).fetch_all(&state.db).await?;
    Ok(Json(Snapshot {
        code: room.code,
        title: room.title,
        role,
        status,
        current_index: room.current_index,
        cues,
        controllers,
        events,
        expires_at: room.expires_at,
        webhook_configured: room.webhook_url.is_some(),
    }))
}

async fn approve_controller(
    Path((code, controller_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Query(query): Query<TokenQuery>,
) -> Result<StatusCode, ApiError> {
    let room = room_by_code(&state.db, &code).await?;
    let (role, _) = role_for(&state.db, &room, &query.token).await?;
    if role != "host" {
        return Err(ApiError::NotFound);
    }
    let changed = sqlx::query("UPDATE controllers SET status = 'approved' WHERE id = ? AND room_id = ? AND status = 'pending'").bind(controller_id).bind(&room.id).execute(&state.db).await?;
    if changed.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    notify(&state, &room.code, "controller_approved").await;
    Ok(StatusCode::NO_CONTENT)
}

async fn reject_controller(
    Path((code, controller_id)): Path<(String, String)>,
    State(state): State<AppState>,
    Query(query): Query<TokenQuery>,
) -> Result<StatusCode, ApiError> {
    let room = room_by_code(&state.db, &code).await?;
    let (role, _) = role_for(&state.db, &room, &query.token).await?;
    if role != "host" {
        return Err(ApiError::NotFound);
    }
    sqlx::query("UPDATE controllers SET status = 'rejected' WHERE id = ? AND room_id = ?")
        .bind(controller_id)
        .bind(&room.id)
        .execute(&state.db)
        .await?;
    notify(&state, &room.code, "controller_rejected").await;
    Ok(StatusCode::NO_CONTENT)
}

async fn fire_next(
    Path(code): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<TokenQuery>,
) -> Result<Json<CueEvent>, ApiError> {
    fire_at(code, state, query.token, None).await
}

async fn set_cue(
    Path(code): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<TokenQuery>,
    Json(input): Json<SetCueInput>,
) -> Result<Json<CueEvent>, ApiError> {
    let room = room_by_code(&state.db, &code).await?;
    let (role, _) = role_for(&state.db, &room, &query.token).await?;
    if role != "host" {
        return Err(ApiError::NotFound);
    }
    fire_at(code, state, query.token, Some(input.index)).await
}

async fn fire_at(
    code: String,
    state: AppState,
    token: String,
    requested: Option<i64>,
) -> Result<Json<CueEvent>, ApiError> {
    let _guard = state.cue_lock.lock().await;
    let room = room_by_code(&state.db, &code).await?;
    let (role, status) = role_for(&state.db, &room, &token).await?;
    if status != "approved" {
        return Err(ApiError::Pending);
    }
    let actor = if role == "host" {
        "Host".to_string()
    } else {
        sqlx::query_scalar::<_, String>(
            "SELECT name FROM controllers WHERE room_id = ? AND token_hash = ?",
        )
        .bind(&room.id)
        .bind(hash_token(&token))
        .fetch_one(&state.db)
        .await?
    };
    let index = requested.unwrap_or(room.current_index + 1);
    let cue = sqlx::query_as::<_, Cue>(
        "SELECT id, position, scene, name FROM cues WHERE room_id = ? AND position = ?",
    )
    .bind(&room.id)
    .bind(index)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| ApiError::BadRequest("The cue list is already complete".into()))?;
    let sequence: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sequence), 0) + 1 FROM cue_events WHERE room_id = ?",
    )
    .bind(&room.id)
    .fetch_one(&state.db)
    .await?;
    let event = CueEvent {
        id: Uuid::new_v4().to_string(),
        sequence,
        cue_id: cue.id.clone(),
        scene: cue.scene.clone(),
        cue_name: cue.name.clone(),
        controller_name: actor,
        webhook_status: if room.webhook_url.is_some() {
            "queued".into()
        } else {
            "not_configured".into()
        },
        created_at: Utc::now().to_rfc3339(),
    };
    let mut tx = state.db.begin().await?;
    sqlx::query("UPDATE rooms SET current_index = ? WHERE id = ?")
        .bind(index)
        .bind(&room.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO cue_events (id, room_id, sequence, cue_id, scene, cue_name, controller_name, webhook_status, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&event.id).bind(&room.id).bind(event.sequence).bind(&event.cue_id).bind(&event.scene).bind(&event.cue_name).bind(&event.controller_name).bind(&event.webhook_status).bind(&event.created_at).execute(&mut *tx).await?;
    tx.commit().await?;
    notify(&state, &room.code, "cue_fired").await;
    if let (Some(url), Some(secret)) = (room.webhook_url, room.webhook_secret) {
        let state_clone = state.clone();
        let room_id = room.id;
        let event_clone = event.clone();
        tokio::spawn(async move {
            deliver_webhook(state_clone, room_id, event_clone, url, secret).await;
        });
    }
    Ok(Json(event))
}

async fn deliver_webhook(
    state: AppState,
    room_id: String,
    event: CueEvent,
    url: String,
    secret: String,
) {
    if validate_public_webhook(&url).is_err() || validate_resolved_webhook(&url).await.is_err() {
        let _ = sqlx::query("UPDATE cue_events SET webhook_status = 'failed' WHERE id = ?")
            .bind(&event.id)
            .execute(&state.db)
            .await;
        return;
    }
    let body = serde_json::to_vec(&serde_json::json!({"type":"cue.fired","event":event}))
        .unwrap_or_default();
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("hmac accepts key");
    mac.update(&body);
    let signature = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(client) => client,
        Err(_) => return,
    };
    let result = client
        .post(url)
        .header("content-type", "application/json")
        .header("x-scene-cues-signature", signature)
        .timeout(std::time::Duration::from_secs(5))
        .body(body)
        .send()
        .await;
    let status = match result {
        Ok(response) if response.status().is_success() => "delivered",
        _ => "failed",
    };
    let _ = sqlx::query("UPDATE cue_events SET webhook_status = ? WHERE id = ?")
        .bind(status)
        .bind(&event.id)
        .execute(&state.db)
        .await;
    if let Ok(code) = sqlx::query_scalar::<_, String>("SELECT code FROM rooms WHERE id = ?")
        .bind(room_id)
        .fetch_one(&state.db)
        .await
    {
        notify(&state, &code, "webhook_updated").await;
    }
}

async fn clear_logs(
    Path(code): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<TokenQuery>,
) -> Result<StatusCode, ApiError> {
    let room = room_by_code(&state.db, &code).await?;
    if role_for(&state.db, &room, &query.token).await?.0 != "host" {
        return Err(ApiError::NotFound);
    }
    sqlx::query("DELETE FROM cue_events WHERE room_id = ?")
        .bind(&room.id)
        .execute(&state.db)
        .await?;
    notify(&state, &room.code, "logs_cleared").await;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_room(
    Path(code): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<TokenQuery>,
) -> Result<StatusCode, ApiError> {
    let room = room_by_code(&state.db, &code).await?;
    if role_for(&state.db, &room, &query.token).await?.0 != "host" {
        return Err(ApiError::NotFound);
    }
    sqlx::query("DELETE FROM rooms WHERE id = ?")
        .bind(&room.id)
        .execute(&state.db)
        .await?;
    notify(&state, &room.code, "room_ended").await;
    Ok(StatusCode::NO_CONTENT)
}

async fn events(
    Path(code): Path<String>,
    State(state): State<AppState>,
    Query(query): Query<TokenQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ApiError> {
    let room = room_by_code(&state.db, &code).await?;
    role_for(&state.db, &room, &query.token).await?;
    let mut channels = state.channels.lock().await;
    let sender = channels
        .entry(room.code.clone())
        .or_insert_with(|| broadcast::channel(64).0)
        .clone();
    let stream = BroadcastStream::new(sender.subscribe()).filter_map(|message| match message {
        Ok(data) => Some(Ok(Event::default().event("refresh").data(data))),
        Err(_) => None,
    });
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn notify(state: &AppState, code: &str, message: &str) {
    let channels = state.channels.lock().await;
    if let Some(sender) = channels.get(code) {
        let _ = sender.send(message.to_string());
    }
}

async fn cleanup_expired(db: &SqlitePool) {
    loop {
        if let Err(err) = sqlx::query("DELETE FROM rooms WHERE expires_at <= ?")
            .bind(Utc::now().to_rfc3339())
            .execute(db)
            .await
        {
            tracing::warn!(error = %err, "expiration cleanup failed");
        }
        tokio::time::sleep(std::time::Duration::from_secs(900)).await;
    }
}

async fn security_headers(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("x-frame-options", HeaderValue::from_static("DENY"));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert("content-security-policy", HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'"));
    if path.starts_with("/assets/") || path.starts_with("/fonts/") {
        headers.insert(
            "cache-control",
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    } else if !path.starts_with("/api/") {
        headers.insert("cache-control", HeaderValue::from_static("no-cache"));
    }
    response
}

fn app(state: AppState, static_dir: PathBuf) -> Router {
    let fallback =
        ServeDir::new(&static_dir).fallback(ServeFile::new(static_dir.join("index.html")));
    let rate_limit = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(30)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("valid rate-limit configuration"),
    );
    let api = Router::new()
        .route("/rooms", post(create_room))
        .route("/rooms/:code", get(snapshot).delete(delete_room))
        .route("/rooms/:code/join", post(join_room))
        .route("/rooms/:code/events", get(events))
        .route("/rooms/:code/fire", post(fire_next))
        .route("/rooms/:code/set", post(set_cue))
        .route("/rooms/:code/logs", delete(clear_logs))
        .route(
            "/rooms/:code/controllers/:controller_id/approve",
            post(approve_controller),
        )
        .route(
            "/rooms/:code/controllers/:controller_id/reject",
            post(reject_controller),
        )
        .layer(DefaultBodyLimit::max(32 * 1024))
        .layer(GovernorLayer { config: rate_limit });
    Router::new()
        .route("/health", get(health))
        .nest("/api", api)
        .fallback_service(fallback)
        .layer(middleware::from_fn(security_headers))
        .layer(CompressionLayer::new())
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    path = %request.uri().path()
                )
            }),
        )
        .with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://scene-cues.db?mode=rwc".into());
    let options = SqliteConnectOptions::from_str(&database_url)
        .expect("valid DATABASE_URL")
        .create_if_missing(true)
        .foreign_keys(true);
    let db = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await
        .expect("connect database");
    sqlx::migrate!().run(&db).await.expect("run migrations");
    let state = AppState {
        db: db.clone(),
        channels: Default::default(),
        cue_lock: Default::default(),
        public_url: std::env::var("PUBLIC_URL").unwrap_or_else(|_| "http://localhost:8080".into()),
    };
    tokio::spawn(async move { cleanup_expired(&db).await });
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("bind port");
    tracing::info!(port, "Scene Cues listening");
    axum::serve(
        listener,
        app(
            state,
            PathBuf::from(std::env::var("STATIC_DIR").unwrap_or_else(|_| "dist".into())),
        )
        .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown())
    .await
    .expect("serve");
}

async fn shutdown() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.expect("ctrl-c handler") };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    async fn test_app() -> Router {
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        app(
            AppState {
                db,
                channels: Default::default(),
                cue_lock: Default::default(),
                public_url: "https://scene.test".into(),
            },
            PathBuf::from("/tmp/scene-cues-missing-static"),
        )
    }

    fn request(method: &str, uri: &str, body: Option<serde_json::Value>) -> Request<Body> {
        let mut request = Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(
                body.map(|value| value.to_string()).unwrap_or_default(),
            ))
            .unwrap();
        request
            .extensions_mut()
            .insert(axum::extract::ConnectInfo(SocketAddr::from((
                [127, 0, 0, 1],
                41000,
            ))));
        request
    }

    async fn json_body(response: Response) -> serde_json::Value {
        serde_json::from_slice(&to_bytes(response.into_body(), 128 * 1024).await.unwrap()).unwrap()
    }

    #[test]
    fn rejects_local_webhooks() {
        assert!(validate_public_webhook("http://example.com/hook").is_err());
        assert!(validate_public_webhook("https://127.0.0.1/hook").is_err());
        assert!(validate_public_webhook("https://example.com/hook").is_ok());
    }
    #[test]
    fn room_codes_are_unambiguous() {
        let code = random_code();
        assert_eq!(code.len(), 6);
        assert!(!code.contains('I') && !code.contains('O'));
    }

    #[tokio::test]
    async fn health_has_an_explicit_build_identity() {
        let response = test_app()
            .await
            .oneshot(request("GET", "/health", None))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let health = json_body(response).await;
        assert_eq!(health["status"], "ok");
        assert_ne!(health["build_sha"], "unknown");
        assert!(!health["build_sha"].as_str().unwrap_or_default().is_empty());
    }

    #[tokio::test]
    async fn response_policy_does_not_advertise_an_unavailable_billing_origin() {
        let response = test_app()
            .await
            .oneshot(request("GET", "/", None))
            .await
            .unwrap();
        let policy = response
            .headers()
            .get("content-security-policy")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        assert!(policy.contains("connect-src 'self'"));
        assert!(!policy.contains("sociobot.in"));
    }

    #[tokio::test]
    async fn rejects_thirteenth_cue_at_the_api_boundary() {
        let router = test_app().await;
        let cues: Vec<_> = (1..=FREE_CUE_LIMIT + 1)
            .map(|number| serde_json::json!({"scene":"Act", "name":format!("Cue {number}")}))
            .collect();
        let response = router
            .oneshot(request(
                "POST",
                "/api/rooms",
                Some(serde_json::json!({
                    "title": "Too many cues",
                    "cues": cues,
                    "webhook_url": null,
                    "webhook_secret": null
                })),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            json_body(response).await["error"],
            "Add between 1 and 12 cues"
        );
    }

    #[tokio::test]
    async fn full_room_approval_and_cue_flow() {
        let router = test_app().await;
        let created = router
            .clone()
            .oneshot(request(
                "POST",
                "/api/rooms",
                Some(serde_json::json!({
                    "title": "Dress rehearsal",
                    "cues": [
                        {"scene":"Opening", "name":"House lights"},
                        {"scene":"Opening", "name":"Music"}
                    ],
                    "webhook_url": null,
                    "webhook_secret": null
                })),
            ))
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::CREATED);
        let created = json_body(created).await;
        let code = created["code"].as_str().unwrap();
        let host_token = created["host_token"].as_str().unwrap();
        let join_url = Url::parse(created["join_url"].as_str().unwrap()).unwrap();
        let secret = join_url
            .query_pairs()
            .find(|(key, _)| key == "secret")
            .unwrap()
            .1
            .into_owned();

        let joined = router
            .clone()
            .oneshot(request(
                "POST",
                &format!("/api/rooms/{code}/join"),
                Some(serde_json::json!({"name":"Booth phone", "secret":secret})),
            ))
            .await
            .unwrap();
        assert_eq!(joined.status(), StatusCode::CREATED);
        let controller_token = json_body(joined).await["token"]
            .as_str()
            .unwrap()
            .to_string();

        let denied = router
            .clone()
            .oneshot(request(
                "POST",
                &format!("/api/rooms/{code}/fire?token={controller_token}"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);

        let host_view = router
            .clone()
            .oneshot(request(
                "GET",
                &format!("/api/rooms/{code}?token={host_token}"),
                None,
            ))
            .await
            .unwrap();
        let host_view = json_body(host_view).await;
        let controller_id = host_view["controllers"][0]["id"].as_str().unwrap();
        let approved = router
            .clone()
            .oneshot(request(
                "POST",
                &format!(
                    "/api/rooms/{code}/controllers/{controller_id}/approve?token={host_token}"
                ),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(approved.status(), StatusCode::NO_CONTENT);

        let fired = router
            .clone()
            .oneshot(request(
                "POST",
                &format!("/api/rooms/{code}/fire?token={controller_token}"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(fired.status(), StatusCode::OK);
        assert_eq!(json_body(fired).await["sequence"], 1);

        let cleared = router
            .clone()
            .oneshot(request(
                "DELETE",
                &format!("/api/rooms/{code}/logs?token={host_token}"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(cleared.status(), StatusCode::NO_CONTENT);
        let ended = router
            .oneshot(request(
                "DELETE",
                &format!("/api/rooms/{code}?token={host_token}"),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(ended.status(), StatusCode::NO_CONTENT);
    }
}
