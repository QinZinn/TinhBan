//! Tinh Bàn — truy cập SQLite + migration.
//!
//! Crate này chứa:
//!  - kết nối pool SQLite qua `sqlx`
//!  - chạy migration (embed qua `sqlx::migrate!("./migrations")` — compile vào
//!    binary, deploy không cần folder migration bên ngoài)
//!  - vài hàm truy vấn mẫu cho bảng `app_meta`
//!  - cache kết quả scrape licham365.vn ([`get_licham365_cache`],
//!    [`put_licham365_ok`], [`put_licham365_error`]) — giai đoạn 5
//!
//! `DATABASE_URL` đọc từ biến môi trường, mặc định `sqlite:data/tinhban.db?mode=rwc`
//! khi dev local (tạo file trong `./data/`). Trên server thật sẽ trỏ tới đường
//! dẫn hệ thống, không hardcode.

pub use sqlx;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};

/// Giá trị mặc định cho `DATABASE_URL` khi không đặt env.
pub const DEFAULT_DATABASE_URL: &str = "sqlite:data/tinhban.db?mode=rwc";

/// Đọc `DATABASE_URL` từ env, hoặc về [`DEFAULT_DATABASE_URL`].
pub fn default_database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string())
}

/// Tạo pool SQLite. Tự tạo thư mục cha của file DB nếu cần (cho đường dẫn tương
/// đối như `data/tinhban.db`).
pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    if let Some(rest) = database_url.strip_prefix("sqlite:") {
        let file_part = rest.split('?').next().unwrap_or(rest);
        if !file_part.is_empty() && file_part != ":memory:" {
            if let Some(parent) = std::path::Path::new(file_part).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
        }
    }

    let opts: SqliteConnectOptions = database_url.parse()?;
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;
    Ok(pool)
}

/// Chạy toàn bộ migration đã embed.
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Tạo pool + chạy migration luôn (gọn cho khởi động server).
pub async fn init_and_migrate(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = init_pool(database_url).await?;
    run_migrations(&pool)
        .await
        .map_err(|e| sqlx::Error::Migrate(Box::new(e)))?;
    Ok(pool)
}

/// Đọc `value` của `key` từ `app_meta`.
pub async fn get_app_meta(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query("SELECT value FROM app_meta WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.get::<String, _>("value")))
}

/// Ghi/overwrite `value` cho `key`. Tự cập nhật `updated_at`.
pub async fn set_app_meta(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO app_meta (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value,
            updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

// ===========================================================================
// Cache licham365.vn (giai đoạn 5)
// ===========================================================================

/// Một bản ghi cache scrape licham365.vn cho một ngày dương lịch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Licham365Cache {
    /// Ngày dương lịch 'YYYY-MM-DD'.
    pub solar_date: String,
    /// `true` nếu lần scrape đó thành công.
    pub ok: bool,
    /// JSON các mục diễn giải — chỉ có khi `ok == true`.
    pub payload: Option<String>,
    /// Mô tả lỗi — chỉ có khi `ok == false`.
    pub error: Option<String>,
    /// URL đã gọi.
    pub source_url: String,
    /// Thời điểm scrape, ISO-8601 UTC.
    pub fetched_at: String,
}

/// Đọc bản ghi cache của `solar_date` ('YYYY-MM-DD'), kể cả bản ghi lỗi.
///
/// Trả `Ok(None)` khi chưa từng scrape ngày đó. Việc quyết định bản ghi lỗi đã
/// "quá cũ" để thử lại hay chưa thuộc về tầng gọi (xem `fetched_at`) — tầng DB
/// cố ý không áp chính sách TTL.
pub async fn get_licham365_cache(
    pool: &SqlitePool,
    solar_date: &str,
) -> Result<Option<Licham365Cache>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT solar_date, status, payload, error, source_url, fetched_at
         FROM licham365_cache WHERE solar_date = ?",
    )
    .bind(solar_date)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Licham365Cache {
        solar_date: r.get::<String, _>("solar_date"),
        ok: r.get::<String, _>("status") == "ok",
        payload: r.get::<Option<String>, _>("payload"),
        error: r.get::<Option<String>, _>("error"),
        source_url: r.get::<String, _>("source_url"),
        fetched_at: r.get::<String, _>("fetched_at"),
    }))
}

/// Ghi (hoặc ghi đè) bản ghi cache **thành công**.
///
/// Ghi đè có chủ đích: nếu trước đó là bản ghi lỗi, lần scrape thành công sau
/// phải thay thế nó — nếu không, một lần site sập sẽ đóng băng ngày đó vĩnh viễn.
pub async fn put_licham365_ok(
    pool: &SqlitePool,
    solar_date: &str,
    payload_json: &str,
    source_url: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO licham365_cache (solar_date, status, payload, error, source_url, fetched_at)
         VALUES (?, 'ok', ?, NULL, ?, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
         ON CONFLICT(solar_date) DO UPDATE SET
            status     = 'ok',
            payload    = excluded.payload,
            error      = NULL,
            source_url = excluded.source_url,
            fetched_at = excluded.fetched_at",
    )
    .bind(solar_date)
    .bind(payload_json)
    .bind(source_url)
    .execute(pool)
    .await?;
    Ok(())
}

/// Ghi (hoặc ghi đè) bản ghi cache **lỗi**.
///
/// KHÔNG ghi đè lên bản ghi `ok` đang có: một lần scrape hỏng không được phép
/// xoá dữ liệu tốt đã lấy được trước đó.
pub async fn put_licham365_error(
    pool: &SqlitePool,
    solar_date: &str,
    error: &str,
    source_url: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO licham365_cache (solar_date, status, payload, error, source_url, fetched_at)
         VALUES (?, 'error', NULL, ?, ?, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
         ON CONFLICT(solar_date) DO UPDATE SET
            status     = 'error',
            error      = excluded.error,
            source_url = excluded.source_url,
            fetched_at = excluded.fetched_at
         WHERE licham365_cache.status = 'error'",
    )
    .bind(solar_date)
    .bind(error)
    .bind(source_url)
    .execute(pool)
    .await?;
    Ok(())
}

/// Xoá các bản ghi **lỗi** cũ hơn `older_than` (chuỗi ISO-8601 UTC), trả về số
/// dòng đã xoá. Bản ghi `ok` không bao giờ bị đụng tới.
pub async fn purge_licham365_errors(
    pool: &SqlitePool,
    older_than: &str,
) -> Result<u64, sqlx::Error> {
    let res = sqlx::query("DELETE FROM licham365_cache WHERE status = 'error' AND fetched_at < ?")
        .bind(older_than)
        .execute(pool)
        .await?;
    Ok(res.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Xác nhận pipeline migration hoạt động: init pool (in-memory) + migrate tạo
    /// bảng `app_meta`, sau đó get/set roundtrip.
    #[tokio::test]
    async fn migrations_and_app_meta_roundtrip() {
        let pool = init_and_migrate("sqlite::memory:")
            .await
            .expect("init + migrate in-memory pool");

        // Lúc chưa ghi, get trả None.
        assert_eq!(get_app_meta(&pool, "name").await.unwrap(), None);

        // Ghi rồi đọc lại.
        set_app_meta(&pool, "name", "Tinh Bàn").await.unwrap();
        assert_eq!(
            get_app_meta(&pool, "name").await.unwrap(),
            Some("Tinh Bàn".to_string())
        );

        // Overwrite.
        set_app_meta(&pool, "name", "v2").await.unwrap();
        assert_eq!(
            get_app_meta(&pool, "name").await.unwrap(),
            Some("v2".to_string())
        );
    }

    /// Cache licham365: ghi thành công → đọc lại đúng; ghi đè lỗi bằng thành
    /// công được phép, nhưng lỗi KHÔNG được ghi đè lên thành công.
    #[tokio::test]
    async fn licham365_cache_roundtrip_va_uu_tien_ban_ghi_tot() {
        let pool = init_and_migrate("sqlite::memory:").await.expect("migrate");
        let d = "2024-03-15";
        let url = "https://licham365.vn/lich-am-ngay-15-thang-3-nam-2024";

        assert_eq!(get_licham365_cache(&pool, d).await.unwrap(), None);

        // Lỗi trước.
        put_licham365_error(&pool, d, "timeout", url).await.unwrap();
        let c = get_licham365_cache(&pool, d).await.unwrap().unwrap();
        assert!(!c.ok);
        assert_eq!(c.error.as_deref(), Some("timeout"));
        assert_eq!(c.payload, None);

        // Thành công ghi đè lên lỗi.
        put_licham365_ok(&pool, d, r#"{"a":1}"#, url).await.unwrap();
        let c = get_licham365_cache(&pool, d).await.unwrap().unwrap();
        assert!(c.ok);
        assert_eq!(c.payload.as_deref(), Some(r#"{"a":1}"#));
        assert_eq!(c.error, None);

        // Lỗi KHÔNG được ghi đè lên thành công.
        put_licham365_error(&pool, d, "site sập", url).await.unwrap();
        let c = get_licham365_cache(&pool, d).await.unwrap().unwrap();
        assert!(c.ok, "bản ghi tốt phải được giữ nguyên khi scrape sau đó hỏng");
        assert_eq!(c.payload.as_deref(), Some(r#"{"a":1}"#));
    }

    /// `purge_licham365_errors` chỉ xoá bản ghi lỗi, không đụng bản ghi tốt.
    #[tokio::test]
    async fn purge_chi_xoa_ban_ghi_loi() {
        let pool = init_and_migrate("sqlite::memory:").await.expect("migrate");
        put_licham365_ok(&pool, "2024-01-01", "{}", "u").await.unwrap();
        put_licham365_error(&pool, "2024-01-02", "boom", "u").await.unwrap();

        // Mốc ở tương lai xa → mọi bản ghi lỗi đều "cũ hơn".
        let n = purge_licham365_errors(&pool, "2999-01-01T00:00:00Z").await.unwrap();
        assert_eq!(n, 1);
        assert!(get_licham365_cache(&pool, "2024-01-01").await.unwrap().is_some());
        assert!(get_licham365_cache(&pool, "2024-01-02").await.unwrap().is_none());
    }
}
