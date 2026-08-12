//! Tinh Bàn — truy cập SQLite + migration.
//!
//! Crate này chứa:
//!  - kết nối pool SQLite qua `sqlx`
//!  - chạy migration (embed qua `sqlx::migrate!("./migrations")` — compile vào
//!    binary, deploy không cần folder migration bên ngoài)
//!  - vài hàm truy vấn mẫu cho bảng `app_meta`
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
}