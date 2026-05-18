use axum::{
    extract::State,
    http::Method,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

// --- [데이터 구조체 정의] ---
// 프론트엔드에서 보내는 지원서 데이터의 형태입니다.
#[derive(Deserialize, Serialize, Debug)]
struct ApplicationForm {
    name: String,
    student_id: String,
    major: String,
    phone: String,
    email: String,
    message: String,
}

// 프론트엔드에서 보내는 문의사항 데이터의 형태입니다.
#[derive(Deserialize, Serialize, Debug)]
struct InquiryForm {
    title: String,
    content: String,
    contact: String,
}

// 데이터베이스 연결 풀을 서버 전체에서 공유하기 위한 구조체입니다.
struct AppState {
    db: SqlitePool,
}

#[tokio::main]
async fn main() {
    // 1. 데이터베이스 연결 및 테이블 생성
    let db_url = "sqlite:club.db";
    let pool = SqlitePool::connect(db_url).await.expect("DB 연결에 실패했습니다.");

    sqlx::query(
        "
        CREATE TABLE IF NOT EXISTS applicants (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            student_id TEXT NOT NULL,
            major TEXT NOT NULL,
            phone TEXT NOT NULL,
            email TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS inquiries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT NOT NULL,
            contact TEXT NOT NULL,
            is_answered INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "
    )
    .execute(&pool)
    .await
    .unwrap();

    let state = Arc::new(AppState { db: pool });

    // 2. CORS (보안) 설정
    let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_origin(Any);

    // 3. 라우터 설정 (주소와 함수를 연결)
    let app = Router::new()
        .route("/apply", post(handle_apply))
        .route("/inquiry", post(handle_inquiry))
        .layer(cors)
        .with_state(state);

    // 4. 서버 실행
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 [동아리 백엔드] 지원/문의 서버가 http://localhost:3000 에서 실행 중입니다.");
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// --- [비즈니스 로직: 핸들러 함수] ---

// 지원서 데이터를 처리하는 함수입니다.
async fn handle_apply(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ApplicationForm>,
) -> Json<serde_json::Value> {
    println!("📝 새 지원서 도착: {} ({})", payload.name, payload.student_id);

    // DB에 데이터를 저장(INSERT)합니다.
    let result = sqlx::query(
        "INSERT INTO applicants (name, student_id, major, phone, email, message) VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&payload.name)
    .bind(&payload.student_id)
    .bind(&payload.major)
    .bind(&payload.phone)
    .bind(&payload.email)
    .bind(&payload.message)
    .execute(&state.db)
    .await;

    // 저장 성공/실패 여부에 따라 프론트엔드에 응답을 보냅니다.
    match result {
        Ok(_) => Json(serde_json::json!({ "status": "success", "message": "지원서 접수가 완료되었습니다." })),
        Err(e) => {
            eprintln!("DB 오류: {}", e);
            Json(serde_json::json!({ "status": "error", "message": "서버 저장 실패" }))
        }
    }
}

// 문의사항 데이터를 처리하는 함수입니다.
async fn handle_inquiry(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InquiryForm>,
) -> Json<serde_json::Value> {
    println!("❓ 새 문의사항 도착: {}", payload.title);

    let result = sqlx::query(
        "INSERT INTO inquiries (title, content, contact) VALUES (?, ?, ?)"
    )
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&payload.contact)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Json(serde_json::json!({ "status": "success", "message": "문의사항이 등록되었습니다." })),
        Err(e) => {
            eprintln!("DB 오류: {}", e);
            Json(serde_json::json!({ "status": "error", "message": "서버 저장 실패" }))
        }
    }
}