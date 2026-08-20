//! Tinh Bàn — server binary.
//!
//! # Kiến trúc
//!
//! **SSR thuần, không JavaScript.** Trang được dựng sẵn ở server bằng `rsx!` rồi
//! render ra chuỗi HTML (`dioxus::ssr::render_element`); form gửi bằng POST
//! thường và server trả redirect 303 (Post/Redirect/Get). Không có wasm, không
//! hydration, không server function — nên `cargo build` là đủ, không cần `dx`
//! CLI hay target `wasm32-unknown-unknown`.
//!
//! Dioxus ở đây đóng vai **thư viện template** (`rsx!`) chứ không phải framework
//! frontend; phần phục vụ HTTP là Axum thuần.
//!
//! # Route
//!
//! Trang HTML:
//!  - `GET  /`                    trang chủ
//!  - `GET  /la-so/moi`           form lập lá số
//!  - `POST /la-so/moi`           lập + lưu, redirect sang hồ sơ vừa tạo
//!  - `GET  /ho-so`               danh sách hồ sơ
//!  - `GET  /ho-so/:id`           chi tiết + lá số Tử Vi & Bát Tự
//!  - `POST /ho-so/:id/sua`       sửa tên / ghi chú
//!  - `POST /ho-so/:id/xoa`       xoá
//!  - `GET  /tu-dien?q=`          tra cứu từ điển
//!  - `GET  /tu-dien/:slug`       chi tiết một mục
//!  - `GET  /ngay-tot-xau?date=`  xem ngày tốt/xấu
//!
//! API JSON:
//!  - `GET    /health`, `GET /api/health`, `GET /api/version`
//!  - `GET    /api/tu-dien?q=`, `GET /api/tu-dien/:slug`
//!  - `POST   /api/ho-so`, `GET /api/ho-so`, `GET /api/ho-so/:id`,
//!    `PATCH /api/ho-so/:id`, `DELETE /api/ho-so/:id`
//!  - `GET    /api/ngay-tot-xau?date=`
//!
//! PORT/IP đọc từ môi trường qua `dioxus::serve`; mặc định 8080.

use dioxus::fullstack::Lazy;

mod ho_so;
mod ngay_tot_xau;
mod routes;
mod scrape;
mod tu_dien;
mod ui;

/// Pool SQLite dùng chung. Lazy init: mở kết nối, chạy migration, rồi nạp từ
/// điển từ nội dung nhúng trong binary.
///
/// Nếu bước nào hỏng thì app fail-to-start và systemd `Restart=on-failure` lo
/// việc thử lại — thà không khởi động còn hơn chạy với DB nửa vời.
static DB: Lazy<sqlx::SqlitePool> = Lazy::new(|| async move {
    let url = tinhban_db::default_database_url();
    tracing::info!("khởi tạo database: {url}");
    let pool = tinhban_db::init_and_migrate(&url).await?;
    tracing::info!("migration xong");

    match tu_dien::nap_vao_db(&pool).await {
        Ok(n) => tracing::info!("từ điển: nạp {n} mục"),
        // Từ điển hỏng không nên chặn cả app — các tính năng khác vẫn dùng được.
        Err(e) => tracing::error!(error = %e, "nạp từ điển thất bại, trang từ điển sẽ trống"),
    }
    Ok::<sqlx::SqlitePool, sqlx::Error>(pool)
});

/// HTTP client dùng chung cho scrape licham365 (tái dùng connection pool + DNS
/// cache thay vì tạo mới mỗi request).
static HTTP: Lazy<reqwest::Client> = Lazy::new(|| async move { scrape::licham365::build_client() });

fn main() {
    // `.env` chỉ tiện cho dev local; trên server systemd nạp qua EnvironmentFile.
    let _ = dotenvy::dotenv();

    // `dioxus::serve` đọc thư mục file tĩnh khi khởi động. App này không có asset
    // nào (CSS nhúng thẳng trong `<head>`), nhưng thư mục vẫn phải tồn tại nếu
    // không sẽ panic — trỏ vào thư mục `public/` rỗng commit kèm repo.
    if std::env::var("DIOXUS_PUBLIC_PATH").is_err() {
        std::env::set_var(
            "DIOXUS_PUBLIC_PATH",
            concat!(env!("CARGO_MANIFEST_DIR"), "/public"),
        );
    }

    use tracing_subscriber::{fmt, EnvFilter};
    let _ = fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
        )
        .with_target(false)
        .try_init();

    tracing::info!(
        "khởi động {} v{}",
        tinhban_core::app_name(),
        tinhban_core::version()
    );

    dioxus::serve(|| async move {
        use dioxus::server::axum::routing::{delete, get, patch, post};
        use dioxus::server::axum::Router;
        use tower_http::cors::CorsLayer;
        use tower_http::trace::TraceLayer;

        // CORS permissive CỐ Ý: app chỉ chạy nội bộ trong tailnet cho đúng một
        // người dùng, không expose ra internet công cộng. Xem README.
        let router = Router::new()
            // --- Trang HTML
            .route("/", get(routes::trang_chu))
            .route("/la-so/moi", get(routes::form_la_so).post(routes::tao_la_so))
            .route("/ho-so", get(routes::danh_sach_ho_so))
            .route("/ho-so/{id}", get(routes::chi_tiet_ho_so))
            .route("/ho-so/{id}/sua", post(routes::sua_ho_so))
            .route("/ho-so/{id}/xoa", post(routes::xoa_ho_so))
            .route("/tu-dien", get(routes::trang_tu_dien))
            .route("/tu-dien/{slug}", get(routes::chi_tiet_tu_dien))
            .route("/ngay-tot-xau", get(routes::trang_ngay_tot_xau))
            // --- API JSON
            .route("/health", get(health_handler))
            .route("/api/health", get(api_health_handler))
            .route("/api/version", get(version_handler))
            .route("/api/tu-dien", get(routes::api_tu_dien))
            .route("/api/tu-dien/{slug}", get(routes::api_tu_dien_chi_tiet))
            .route(
                "/api/ho-so",
                get(routes::api_danh_sach_ho_so).post(routes::api_tao_ho_so),
            )
            .route("/api/ho-so/{id}", get(routes::api_chi_tiet_ho_so))
            .route("/api/ho-so/{id}", patch(routes::api_sua_ho_so))
            .route("/api/ho-so/{id}", delete(routes::api_xoa_ho_so))
            .route("/api/ngay-tot-xau", get(ngay_tot_xau_handler))
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::very_permissive());

        Ok(router)
    });
}

// ===========================================================================
// Handler nhỏ
// ===========================================================================

async fn health_handler() -> dioxus::server::axum::Json<serde_json::Value> {
    dioxus::server::axum::Json(serde_json::json!({ "status": "ok" }))
}

/// Healthcheck chi tiết: có chạm DB thật nên phát hiện được DB hỏng.
async fn api_health_handler() -> dioxus::server::axum::Json<serde_json::Value> {
    let db_ok = sqlx::query("SELECT 1").execute(&*DB).await.is_ok();
    let so_muc = tinhban_db::tat_ca_tu_dien(&DB).await.map(|v| v.len()).unwrap_or(0);
    dioxus::server::axum::Json(serde_json::json!({
        "status": "ok",
        "db": if db_ok { "ok" } else { "fail" },
        "tu_dien_muc": so_muc,
        "version": tinhban_core::version(),
        "app": tinhban_core::app_name(),
    }))
}

async fn version_handler() -> dioxus::server::axum::Json<serde_json::Value> {
    dioxus::server::axum::Json(serde_json::json!({
        "name": tinhban_core::app_name(),
        "version": tinhban_core::version(),
    }))
}

/// `GET /api/ngay-tot-xau?date=YYYY-MM-DD` — giữ nguyên hợp đồng từ giai đoạn 5.
///
/// Thiếu `date` → hôm nay theo giờ VN. `200` kể cả khi scrape hỏng (khi đó
/// `dien_giai` là `null` kèm `ghi_chu`); `400` khi `date` sai hoặc ngoài phạm vi.
async fn ngay_tot_xau_handler(
    dioxus::server::axum::extract::Query(q): dioxus::server::axum::extract::Query<NgayTotXauQuery>,
) -> Result<
    dioxus::server::axum::Json<ngay_tot_xau::NgayTotXauResponse>,
    (
        dioxus::server::axum::http::StatusCode,
        dioxus::server::axum::Json<serde_json::Value>,
    ),
> {
    use dioxus::server::axum::http::StatusCode;
    use dioxus::server::axum::Json;

    let bad = |msg: String| (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": msg })));

    let date = match q.date.as_deref() {
        Some(s) => chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|_| bad(format!("tham số `date` không hợp lệ: {s:?} — cần YYYY-MM-DD")))?,
        None => (chrono::Utc::now() + chrono::Duration::hours(7)).date_naive(),
    };

    ngay_tot_xau::xem_ngay(&DB, &HTTP, date)
        .await
        .map(Json)
        .map_err(|e| bad(e.to_string()))
}

#[derive(Debug, serde::Deserialize)]
struct NgayTotXauQuery {
    date: Option<String>,
}
