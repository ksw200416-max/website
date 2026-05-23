use axum::{
    extract::{Query, State},
    http::Method,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Row}; // Row와 SqlitePool 임포트!
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

// [데이터 양식]
#[derive(Deserialize)]
struct ApplicationForm {
    name: String,
    student_id: String,
    major: String,
    phone: String,
    email: String,
    message: String,
}

#[derive(Deserialize)]
struct InquiryForm {
    title: String,
    content: String,
    contact: String,
}

#[derive(Deserialize)]
struct AdminQuery {
    uid: i64, 
}

struct AppState {
    db: SqlitePool,
}

// [프로그램 시작점: main 함수]
#[tokio::main]
async fn main() {
    let db_url = "sqlite://club.db?mode=rwc";
    let pool = SqlitePool::connect(db_url).await.expect("DB 연결 실패");

    // 데이터베이스 테이블 자동 생성
    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS users ( id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, gid INTEGER NOT NULL );
        CREATE TABLE IF NOT EXISTS applicants ( id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, student_id TEXT NOT NULL, major TEXT NOT NULL, phone TEXT NOT NULL, email TEXT NOT NULL, message TEXT NOT NULL );
        CREATE TABLE IF NOT EXISTS inquiries ( id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, content TEXT NOT NULL, contact TEXT NOT NULL );
        "
    ).execute(&pool).await.unwrap();

    // 테스트용 운영진 데이터
    let _ = sqlx::query("INSERT OR IGNORE INTO users (id, name, gid) VALUES (1, '운영진철수', 2)").execute(&pool).await;

    let state = Arc::new(AppState { db: pool });
    let cors = CorsLayer::new().allow_methods([Method::GET, Method::POST]).allow_origin(Any).allow_headers(Any);

    let app = Router::new()
        .route("/apply", post(handle_apply))
        .route("/applicants", get(handle_get_applicants))
        .route("/inquiry", post(handle_inquiry))
        .route("/inquiries", get(handle_get_inquiries))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🚀 [지원서/문의하기 서버] 정상적으로 실행되었습니다!");
    axum::serve(listener, app).await.unwrap();
}

// [API 핸들러 함수들]
async fn handle_apply(State(state): State<Arc<AppState>>, Json(payload): Json<ApplicationForm>) -> Json<serde_json::Value> {
    let result = sqlx::query("INSERT INTO applicants (name, student_id, major, phone, email, message) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&payload.name).bind(&payload.student_id).bind(&payload.major).bind(&payload.phone).bind(&payload.email).bind(&payload.message)
        .execute(&state.db).await;

    match result {
        Ok(_) => Json(serde_json::json!({ "status": "success", "message": "지원서가 접수되었습니다." })),
        Err(_) => Json(serde_json::json!({ "status": "error", "message": "저장 실패" }))
    }
}

async fn handle_get_applicants(State(state): State<Arc<AppState>>, Query(params): Query<AdminQuery>) -> Json<serde_json::Value> {
    let gid: Option<i64> = sqlx::query_scalar("SELECT gid FROM users WHERE id = ?").bind(params.uid).fetch_optional(&state.db).await.unwrap_or(None);
    if gid != Some(2) { return Json(serde_json::json!({ "status": "error", "message": "권한 없음" })); }
    
    let rows = sqlx::query("SELECT id, name, student_id, major FROM applicants").fetch_all(&state.db).await.unwrap();
    let list: Vec<_> = rows.into_iter().map(|r| {
        serde_json::json!({
            "id": r.get::<i64, _>("id"),
            "name": r.get::<String, _>("name"),
            "student_id": r.get::<String, _>("student_id"),
            "major": r.get::<String, _>("major")
        })
    }).collect();
    Json(serde_json::json!({ "status": "success", "data": list }))
}

async fn handle_inquiry(State(state): State<Arc<AppState>>, Json(payload): Json<InquiryForm>) -> Json<serde_json::Value> {
    let result = sqlx::query("INSERT INTO inquiries (title, content, contact) VALUES (?, ?, ?)")
        .bind(&payload.title).bind(&payload.content).bind(&payload.contact).execute(&state.db).await;

    match result {
        Ok(_) => Json(serde_json::json!({ "status": "success", "message": "문의 접수 완료" })),
        Err(_) => Json(serde_json::json!({ "status": "error", "message": "저장 실패" }))
    }
}

async fn handle_get_inquiries(State(state): State<Arc<AppState>>, Query(params): Query<AdminQuery>) -> Json<serde_json::Value> {
    let gid: Option<i64> = sqlx::query_scalar("SELECT gid FROM users WHERE id = ?").bind(params.uid).fetch_optional(&state.db).await.unwrap_or(None);
    if gid != Some(2) { return Json(serde_json::json!({ "status": "error", "message": "권한 없음" })); }
    
    let rows = sqlx::query("SELECT id, title, contact FROM inquiries").fetch_all(&state.db).await.unwrap();
    let list: Vec<_> = rows.into_iter().map(|r| {
        serde_json::json!({
            "id": r.get::<i64, _>("id"),
            "title": r.get::<String, _>("title"),
            "contact": r.get::<String, _>("contact")
        })
    }).collect();
    Json(serde_json::json!({ "status": "success", "data": list }))
}