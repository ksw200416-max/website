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

// 1. 지원자 데이터 구조 정의
#[derive(Deserialize, Serialize, Debug)]
struct Application {
    name: String,
    student_id: String,
    major: String,
    message: String,
}

// 2. 데이터베이스 풀을 공유하기 위한 서버 상태 구조체
struct AppState {
    db: SqlitePool,
}

#[tokio::main]
async fn main() {
    // SQLite 데이터베이스 연결 주소 설정
    // 데이터 저장공간 확보하기
    let db_url = "sqlite:applicants.db";
    let pool = SqlitePool::connect(db_url).await.expect("DB 연결에 실패했습니다.");

    // 구버전 sqlx 문법에 맞춘 테이블 생성 쿼리
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS applicants (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            student_id TEXT NOT NULL,
            major TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )"
    )
    .execute(&pool)
    .await
    .unwrap();

    let state = Arc::new(AppState { db: pool });

    // CORS 보안 설정
    let cors = CorsLayer::new()
        .allow_methods([Method::POST])
        .allow_origin(Any);

    // 라우터 설정 및 상태(State) 연결  (외부의 요청 받을 준비)
    let app = Router::new()
        .route("/apply", post(handle_apply))
        .layer(cors)
        .with_state(state);

    // axum 0.6 버전의 서버 실행 방식 (기존 axum::serve 대신 사용)
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 서버가 http://localhost:3000 에서 백엔드 대기 중입니다.");
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// 3. 지원서 처리 데이터베이스 저장 핸들러
async fn handle_apply(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Application>,
) -> Json<serde_json::Value> {
    println!("새로운 지원 접수: {} ({})", payload.name, payload.student_id);

    // sqlx 0.6 버전에 맞춘 대입(Binding) 쿼리
    let result = sqlx::query(
        "INSERT INTO applicants (name, student_id, major, message) VALUES (?, ?, ?, ?)"
    )
    .bind(&payload.name)
    .bind(&payload.student_id)
    .bind(&payload.major)
    .bind(&payload.message)
    .execute(&state.db)
    .await;

    match result {
        Ok(_) => Json(serde_json::json!({ "status": "success", "message": "접수 완료!" })),
        Err(e) => {
            eprintln!("DB 오류: {}", e);
            Json(serde_json::json!({ "status": "error", "message": "저장 중 오류 발생" }))
        }
    }
}