//! HTTP-level integration tests: drive the real Axum router over an isolated
//! Postgres (via `#[sqlx::test]`), exercising the cookie-session auth and the
//! full login → edit → audit flow. Uses `tower::ServiceExt::oneshot`.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use core_infra::seed::{ensure_seeded, sole_org_id, SeedConfig};
use core_infra::Argon2Hasher;
use core_web::{build_router, AppState, WebConfig};
use serde_json::{json, Value};
use sqlx::PgPool;
use time::Duration;
use tower::ServiceExt;

fn test_cfg() -> WebConfig {
    WebConfig {
        bind_addr: "127.0.0.1:0".into(),
        web_origin: "http://localhost:5173".into(),
        cookie_name: "core_sid".into(),
        cookie_secure: false,
        session_ttl: Duration::days(7),
    }
}

async fn app_with_seed(pool: PgPool) -> axum::Router {
    let hasher = Argon2Hasher;
    ensure_seeded(
        &pool,
        &hasher,
        &SeedConfig {
            org_name: "MadeSpace Studio".into(),
            org_domain: Some("madespace.co".into()),
            owner_email: "elena@madespace.co".into(),
            owner_name: "Elena Marchetti".into(),
            owner_password: "owner-password-123!".into(),
        },
    )
    .await
    .unwrap();
    let org = sole_org_id(&pool).await.unwrap().unwrap();
    build_router(AppState::new(pool, test_cfg(), org))
}

async fn body_json(resp: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

fn get(uri: &str, cookie: Option<&str>) -> Request<Body> {
    let mut b = Request::builder().uri(uri).method("GET");
    if let Some(c) = cookie {
        b = b.header(header::COOKIE, c);
    }
    b.body(Body::empty()).unwrap()
}

fn json_req(method: &str, uri: &str, cookie: Option<&str>, body: Value) -> Request<Body> {
    let mut b = Request::builder()
        .uri(uri)
        .method(method)
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(c) = cookie {
        b = b.header(header::COOKIE, c);
    }
    b.body(Body::from(body.to_string())).unwrap()
}

/// Log in as the seeded owner and return the session cookie string.
async fn login(app: &axum::Router) -> String {
    let resp = app
        .clone()
        .oneshot(json_req(
            "POST",
            "/api/auth/login",
            None,
            json!({ "email": "elena@madespace.co", "password": "owner-password-123!" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "login should succeed");
    let set_cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .unwrap()
        .to_str()
        .unwrap();
    // "core_sid=<value>; HttpOnly; ..." → keep just the name=value part.
    set_cookie.split(';').next().unwrap().to_string()
}

#[sqlx::test(migrations = "../../migrations")]
async fn unauthenticated_request_is_401(pool: PgPool) {
    let app = app_with_seed(pool).await;
    let resp = app
        .oneshot(get("/api/permissions/matrix", None))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_wrong_password_is_401(pool: PgPool) {
    let app = app_with_seed(pool).await;
    let resp = app
        .oneshot(json_req(
            "POST",
            "/api/auth/login",
            None,
            json!({ "email": "elena@madespace.co", "password": "nope" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "../../migrations")]
async fn login_then_me_returns_permissions(pool: PgPool) {
    let app = app_with_seed(pool).await;
    let cookie = login(&app).await;
    let resp = app
        .clone()
        .oneshot(get("/api/auth/me", Some(&cookie)))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let me = body_json(resp).await;
    assert_eq!(me["user"]["email"], "elena@madespace.co");
    assert_eq!(me["user"]["roleKey"], "owner");
    let perms = me["permissions"].as_array().unwrap();
    assert!(
        perms.iter().any(|p| p == "org.delete"),
        "owner can delete org"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn cycle_cell_then_audit_records_it(pool: PgPool) {
    let app = app_with_seed(pool).await;
    let cookie = login(&app).await;

    // load matrix to find the admin column role id
    let matrix = body_json(
        app.clone()
            .oneshot(get("/api/permissions/matrix", Some(&cookie)))
            .await
            .unwrap(),
    )
    .await;
    let admin_id = matrix["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["key"] == "admin")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // cycle users.invite for admin (Allow → Scope)
    let resp = app
        .clone()
        .oneshot(json_req(
            "PATCH",
            "/api/permissions/matrix/cell",
            Some(&cookie),
            json!({ "actionKey": "users.invite", "roleId": admin_id }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let cell = body_json(resp).await;
    assert_eq!(cell["state"], "scope");

    // the audit log now contains a permission.edit event
    let audit = body_json(
        app.clone()
            .oneshot(get("/api/audit", Some(&cookie)))
            .await
            .unwrap(),
    )
    .await;
    let actions: Vec<&str> = audit
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["action"].as_str().unwrap())
        .collect();
    assert!(
        actions.contains(&"permission.edit"),
        "audit recorded the edit: {actions:?}"
    );
}

#[sqlx::test(migrations = "../../migrations")]
async fn locked_cell_is_rejected_409(pool: PgPool) {
    let app = app_with_seed(pool).await;
    let cookie = login(&app).await;
    let matrix = body_json(
        app.clone()
            .oneshot(get("/api/permissions/matrix", Some(&cookie)))
            .await
            .unwrap(),
    )
    .await;
    let admin_id = matrix["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["key"] == "admin")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    // org.delete is owner-only → locked for admin
    let resp = app
        .oneshot(json_req(
            "PATCH",
            "/api/permissions/matrix/cell",
            Some(&cookie),
            json!({ "actionKey": "org.delete", "roleId": admin_id }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
    let body = body_json(resp).await;
    assert_eq!(body["error"], "cell_locked");
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_api_key_returns_token_once(pool: PgPool) {
    let app = app_with_seed(pool).await;
    let cookie = login(&app).await;
    let resp = app
        .oneshot(json_req(
            "POST",
            "/api/keys",
            Some(&cookie),
            json!({ "name": "Booking sync", "scopes": ["users.view", "audit.view"] }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let created = body_json(resp).await;
    assert!(created["token"].as_str().unwrap().starts_with("ms_live_"));
    assert_eq!(created["key"]["status"], "active");
    // the listing never exposes the token
    assert!(created["key"].get("token").is_none());
}

#[sqlx::test(migrations = "../../migrations")]
async fn invite_user_appears_in_listing(pool: PgPool) {
    let app = app_with_seed(pool).await;
    let cookie = login(&app).await;
    let resp = app
        .clone()
        .oneshot(json_req(
            "POST",
            "/api/users/invite",
            Some(&cookie),
            json!({ "email": "aoife@madespace.co", "role": "member", "scope": "Notting Hill" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let invited = body_json(resp).await;
    assert_eq!(invited["status"], "invited");
    assert_eq!(invited["roleKey"], "member");

    let users = body_json(app.oneshot(get("/api/users", Some(&cookie))).await.unwrap()).await;
    let emails: Vec<&str> = users
        .as_array()
        .unwrap()
        .iter()
        .map(|u| u["email"].as_str().unwrap())
        .collect();
    assert!(emails.contains(&"aoife@madespace.co"));
}

#[sqlx::test(migrations = "../../migrations")]
async fn manually_created_user_can_sign_in(pool: PgPool) {
    let app = app_with_seed(pool).await;
    let cookie = login(&app).await;

    let resp = app
        .clone()
        .oneshot(json_req(
            "POST",
            "/api/users",
            Some(&cookie),
            json!({
                "name": "Marcus Webb",
                "email": "marcus@madespace.co",
                "role": "manager",
                "scope": "Hampstead",
                "password": "Sufficient1!"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let created = body_json(resp).await;
    assert_eq!(created["status"], "active");
    assert_eq!(created["roleKey"], "manager");
    assert_eq!(created["name"], "Marcus Webb");

    // the manually-created user can immediately sign in with the set password
    let resp = app
        .oneshot(json_req(
            "POST",
            "/api/auth/login",
            None,
            json!({ "email": "marcus@madespace.co", "password": "Sufficient1!" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "../../migrations")]
async fn create_user_rejects_weak_password_422(pool: PgPool) {
    let app = app_with_seed(pool).await;
    let cookie = login(&app).await;
    let resp = app
        .oneshot(json_req(
            "POST",
            "/api/users",
            Some(&cookie),
            json!({ "name": "Weak", "email": "weak@madespace.co", "role": "member", "password": "short" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body = body_json(resp).await;
    assert_eq!(body["error"], "password_policy");
}
