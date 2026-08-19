//! Tinh Bàn — server binary.
//!
//! Kiến trúc frontend: dùng **Dioxus fullstack ở chế độ SSR-only** (không build
//! client/wasm). Lý do chọn + cách bật thêm client hydration sau xem trong
//! `README.md` ở gốc repo. Nhờ SSR-only:
//!  - chỉ 1 binary native, không cần `wasm32-unknown-unknown` target hay `dx` CLI
//!    để `cargo build`/`cargo run`.
//!  - frontend chạy chung server với Axum → đúng 1 service systemd.
//!
//! Endpoints (ngoài SSR homepage ở `/`):
//!  - `GET /health`        -> `{"status":"ok"}`        (healthcheck cho systemd)
//!  - `GET /api/health`    -> chi tiết + check DB       (server fn, frontend gọi)
//!  - `GET /api/version`   -> `{"name","version"}`
//!  - `GET /api/ngay-tot-xau?date=YYYY-MM-DD` -> ngày tốt/xấu (giai đoạn 5)
//!
//! PORT/IP bind do Dioxus (`dioxus::serve`) đọc từ môi trường; mặc định 8080.

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use tinhban_core::{app_name, version};

#[cfg(feature = "server")]
mod ngay_tot_xau;
#[cfg(feature = "server")]
mod scrape;

#[cfg(feature = "server")]
use dioxus::fullstack::Lazy;

/// Pool SQLite dùng chung cả server (lazy init chạy + migrate ở lần truy cập đầu).
/// `Lazy` block thread cho tới khi init xong; nếu init lỗi, app fail-to-start và
/// systemd `Restart=on-failure` lo việc thử lại.
#[cfg(feature = "server")]
static DB: Lazy<sqlx::SqlitePool> = Lazy::new(|| async move {
    let url = tinhban_db::default_database_url();
    tracing::info!("initializing database: {url}");
    let pool = tinhban_db::init_and_migrate(&url).await?;
    tracing::info!("database ready");
    Ok::<sqlx::SqlitePool, sqlx::Error>(pool)
});

/// HTTP client dùng chung cho scrape. Tạo một lần để tái dùng connection pool
/// + DNS cache; tạo mới mỗi request là lãng phí và dễ cạn socket.
#[cfg(feature = "server")]
static HTTP: Lazy<reqwest::Client> = Lazy::new(|| async move {
    scrape::licham365::build_client()
});

// PartialEq + Deserialize cần cho codec server fn + `use_loader`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct HealthStatus {
    status: String,
    db: String,
    version: String,
    app: String,
}

/// `GET /api/health` — server fn (tự đăng ký route + callable trong component).
#[get("/api/health")]
async fn get_health() -> Result<HealthStatus, HttpError> {
    let db_ok = sqlx::query("SELECT 1")
        .execute(&*DB)
        .await
        .is_ok();
    Ok(HealthStatus {
        status: "ok".to_string(),
        db: if db_ok { "ok" } else { "fail" }.to_string(),
        version: version().to_string(),
        app: app_name().to_string(),
    })
}

/// Trang chủ: SSR-render tên app + trạng thái backend (gọi `/api/health`).
#[component]
fn Home() -> Element {
    let health = use_loader(get_health)?.read().clone();

    rsx! {
        div { style: "font-family: system-ui, sans-serif; max-width: 40rem; margin: 2rem auto; padding: 1rem;",
            h1 { "{app_name()}" }
            p { "Toolkit tử vi cá nhân — self-hosted." }

            hr {}

            h2 { "Trạng thái backend" }
            ul {
                li { "API health: {health.status}" }
                li { "Database: {health.db}" }
                li { "Version: {health.version} ({health.app})" }
            }

            p { style: "color:#666; font-size:0.9rem;",
                "Endpoints: GET /health · GET /api/health · GET /api/version · GET /api/ngay-tot-xau?date=YYYY-MM-DD"
            }
        }
    }
}

fn app() -> Element {
    rsx! { Home {} }
}

fn main() {
    #[cfg(feature = "server")]
    {
        // `.env` chỉ tiện cho dev local. Trên server systemd dựng qua EnvironmentFile.
        let _ = dotenvy::dotenv();

        // SSR-only không có `dx` bundle, nên `dioxus-server` sẽ panic khi đọc thư
        // mục mặc định `<exe>/public` (không tồn tại). Trỏ vào thư mục `public`
        // rỗng của crate (commit cùng repo) khi env chưa đặt. Trên deploy có thể
        // override `DIOXUS_PUBLIC_PATH` trong EnvironmentFile.
        if std::env::var("DIOXUS_PUBLIC_PATH").is_err() {
            std::env::set_var(
                "DIOXUS_PUBLIC_PATH",
                concat!(env!("CARGO_MANIFEST_DIR"), "/public"),
            );
        }

        // Logging ra stdout (journald tự thu). `try_init` để không panic nếu Dioxus
        // cũng cài subscriber — nhưng vì gọi trước `dioxus::serve`, subscriber này
        // được đăng ký trước và thắng default format.
        use tracing_subscriber::{EnvFilter, fmt};
        let _ = fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
            )
            .with_target(false)
            .try_init();

        tracing::info!("starting {} v{}", app_name(), version());
    }

    #[cfg(not(feature = "server"))]
    dioxus::launch(app);

    #[cfg(feature = "server")]
    dioxus::serve(|| async move {
        use dioxus::server::axum::routing::get;
        use tower_http::cors::CorsLayer;
        use tower_http::trace::TraceLayer;

        // CORS cấu hình permissive CỐ Ý: app chỉ chạy nội bộ qua Tailscale cho 1
        // người dùng, không expose ra public internet. Xem README mục thiết kế.
        // TraceLayer log request ra tracing -> journalctl thấy được traffic.
        let router = dioxus::server::router(app)
            .route("/health", get(health_handler))
            .route("/api/version", get(version_handler))
            .route("/api/ngay-tot-xau", get(ngay_tot_xau_handler))
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::very_permissive());

        Ok(router)
    });
}

#[cfg(feature = "server")]
async fn health_handler() -> dioxus::server::axum::Json<serde_json::Value> {
    dioxus::server::axum::Json(serde_json::json!({ "status": "ok" }))
}

#[cfg(feature = "server")]
async fn version_handler() -> dioxus::server::axum::Json<serde_json::Value> {
    dioxus::server::axum::Json(serde_json::json!({
        "name": tinhban_core::app_name(),
        "version": tinhban_core::version(),
    }))
}

/// `GET /api/ngay-tot-xau?date=YYYY-MM-DD`
///
/// `date` không bắt buộc — thiếu thì lấy **hôm nay theo giờ Việt Nam** (không
/// phải giờ hệ thống: server có thể chạy UTC, nhưng lịch âm luôn tính theo UTC+7).
///
/// Mã trạng thái:
///  - `200` — thành công (kể cả khi phần diễn giải scrape hỏng: khi đó
///    `dien_giai` là `null` và `ghi_chu` giải thích lý do);
///  - `400` — `date` sai định dạng hoặc ngoài phạm vi 1900–2100.
///
/// Nguồn phụ hỏng KHÔNG bao giờ thành 5xx — đó là chủ ý thiết kế.
#[cfg(feature = "server")]
async fn ngay_tot_xau_handler(
    dioxus::server::axum::extract::Query(q): dioxus::server::axum::extract::Query<
        NgayTotXauQuery,
    >,
) -> Result<
    dioxus::server::axum::Json<ngay_tot_xau::NgayTotXauResponse>,
    (
        dioxus::server::axum::http::StatusCode,
        dioxus::server::axum::Json<serde_json::Value>,
    ),
> {
    use dioxus::server::axum::http::StatusCode;
    use dioxus::server::axum::Json;

    let bad = |msg: String| {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": msg })),
        )
    };

    let date = match q.date.as_deref() {
        Some(s) => chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").map_err(|_| {
            bad(format!(
                "tham số `date` không hợp lệ: {s:?} — cần định dạng YYYY-MM-DD"
            ))
        })?,
        // Hôm nay theo giờ VN (UTC+7), không theo timezone của máy chủ.
        None => (chrono::Utc::now() + chrono::Duration::hours(7)).date_naive(),
    };

    match ngay_tot_xau::xem_ngay(&DB, &HTTP, date).await {
        Ok(r) => Ok(Json(r)),
        Err(e) => Err(bad(e.to_string())),
    }
}

/// Query string của `/api/ngay-tot-xau`.
#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
struct NgayTotXauQuery {
    /// `YYYY-MM-DD`. Thiếu → hôm nay theo giờ VN.
    date: Option<String>,
}
