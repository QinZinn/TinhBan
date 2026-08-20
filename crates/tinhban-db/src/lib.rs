//! Tinh Bàn — truy cập SQLite + migration.
//!
//! Crate này chứa:
//!  - kết nối pool SQLite qua `sqlx`
//!  - chạy migration (embed qua `sqlx::migrate!("./migrations")` — compile vào
//!    binary, deploy không cần folder migration bên ngoài)
//!  - vài hàm truy vấn mẫu cho bảng `app_meta`
//!  - cache kết quả scrape licham365.vn ([`get_licham365_cache`],
//!    [`put_licham365_ok`], [`put_licham365_error`]) — giai đoạn 5
//!  - từ điển tử vi + tìm kiếm toàn văn FTS5 ([`nap_tu_dien`], [`tim_tu_dien`],
//!    [`lay_tu_dien`]) và hồ sơ người đã xem ([`them_ho_so`], [`danh_sach_ho_so`],
//!    …) — giai đoạn 6
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

// ===========================================================================
// Chuẩn hoá tiếng Việt cho tìm kiếm (giai đoạn 6)
// ===========================================================================

/// Bỏ dấu tiếng Việt + thường hoá, để tìm kiếm không dấu hoạt động.
///
/// Vì sao tự làm thay vì dùng `remove_diacritics 2` của FTS5: tokenizer của
/// SQLite bỏ được dấu thanh (á → a) nhưng **không** map `đ` → `d`, vì `đ` là một
/// chữ cái riêng chứ không phải `d` cộng dấu. Không xử lý thì gõ "dao hoa" sẽ
/// không tìm ra "Đào Hoa".
///
/// Hàm này map cả nguyên âm có dấu lẫn `đ`, nên tra cứu không dấu đều cho mọi
/// chữ. Ký tự không phải chữ/số biến thành khoảng trắng để FTS5 tách token sạch.
pub fn chuan_hoa_tim_kiem(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        let lower = ch.to_lowercase().next().unwrap_or(ch);
        let mapped = match lower {
            'à' | 'á' | 'ạ' | 'ả' | 'ã' | 'â' | 'ầ' | 'ấ' | 'ậ' | 'ẩ' | 'ẫ' | 'ă'
            | 'ằ' | 'ắ' | 'ặ' | 'ẳ' | 'ẵ' => 'a',
            'è' | 'é' | 'ẹ' | 'ẻ' | 'ẽ' | 'ê' | 'ề' | 'ế' | 'ệ' | 'ể' | 'ễ' => 'e',
            'ì' | 'í' | 'ị' | 'ỉ' | 'ĩ' => 'i',
            'ò' | 'ó' | 'ọ' | 'ỏ' | 'õ' | 'ô' | 'ồ' | 'ố' | 'ộ' | 'ổ' | 'ỗ' | 'ơ'
            | 'ờ' | 'ớ' | 'ợ' | 'ở' | 'ỡ' => 'o',
            'ù' | 'ú' | 'ụ' | 'ủ' | 'ũ' | 'ư' | 'ừ' | 'ứ' | 'ự' | 'ử' | 'ữ' => 'u',
            'ỳ' | 'ý' | 'ỵ' | 'ỷ' | 'ỹ' => 'y',
            'đ' => 'd',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        };
        out.push(mapped);
    }
    // Gộp khoảng trắng thừa.
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

// ===========================================================================
// Từ điển tử vi (giai đoạn 6)
// ===========================================================================

/// Một mục từ điển.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MucTuDien {
    pub slug: String,
    pub title: String,
    /// `chinh-tinh` | `phu-tinh` | `cung`.
    pub kind: String,
    pub nhom: String,
    pub nguhanh: String,
    pub amduong: String,
    pub aliases: String,
    /// Thân bài markdown (chưa render).
    pub body: String,
    pub ord: i64,
}

/// Nạp lại **toàn bộ** từ điển: xoá sạch rồi ghi mới trong một transaction.
///
/// Nạp lại toàn bộ thay vì upsert từng mục để bản ghi của mục đã bị xoá khỏi
/// `content/` không còn sót lại. 39 mục nên chi phí không đáng kể.
pub async fn nap_tu_dien(pool: &SqlitePool, muc: &[MucTuDien]) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("DELETE FROM tu_dien").execute(&mut *tx).await?;
    sqlx::query("DELETE FROM tu_dien_fts").execute(&mut *tx).await?;

    for m in muc {
        sqlx::query(
            "INSERT INTO tu_dien (slug, title, kind, nhom, nguhanh, amduong, aliases, body, ord)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&m.slug)
        .bind(&m.title)
        .bind(&m.kind)
        .bind(&m.nhom)
        .bind(&m.nguhanh)
        .bind(&m.amduong)
        .bind(&m.aliases)
        .bind(&m.body)
        .bind(m.ord)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO tu_dien_fts (slug, title_norm, aliases_norm, body_norm)
             VALUES (?, ?, ?, ?)",
        )
        .bind(&m.slug)
        .bind(chuan_hoa_tim_kiem(&m.title))
        .bind(chuan_hoa_tim_kiem(&m.aliases))
        .bind(chuan_hoa_tim_kiem(&m.body))
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

fn map_muc(r: &sqlx::sqlite::SqliteRow) -> MucTuDien {
    MucTuDien {
        slug: r.get("slug"),
        title: r.get("title"),
        kind: r.get("kind"),
        nhom: r.get("nhom"),
        nguhanh: r.get("nguhanh"),
        amduong: r.get("amduong"),
        aliases: r.get("aliases"),
        body: r.get("body"),
        ord: r.get("ord"),
    }
}

/// Lấy một mục theo `slug`.
pub async fn lay_tu_dien(
    pool: &SqlitePool,
    slug: &str,
) -> Result<Option<MucTuDien>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM tu_dien WHERE slug = ?")
        .bind(slug)
        .fetch_optional(pool)
        .await?;
    Ok(row.as_ref().map(map_muc))
}

/// Toàn bộ mục, sắp theo `kind` rồi `ord` — dùng cho trang từ điển khi chưa gõ
/// gì.
pub async fn tat_ca_tu_dien(pool: &SqlitePool) -> Result<Vec<MucTuDien>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT * FROM tu_dien
         ORDER BY CASE kind WHEN 'chinh-tinh' THEN 0 WHEN 'phu-tinh' THEN 1 ELSE 2 END,
                  ord, title",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.iter().map(map_muc).collect())
}

/// Tìm kiếm toàn văn. `q` được chuẩn hoá trước, nên gõ có dấu hay không đều ra.
///
/// Mỗi từ trong `q` thành một tiền tố (`tu*`) và nối bằng `AND`, nên "tu vi" tìm
/// được "Tử Vi" mà "tu" một mình cũng ra. Kết quả sắp theo `bm25` của FTS5, ưu
/// tiên khớp ở tiêu đề hơn ở thân bài.
///
/// `q` rỗng hoặc chỉ toàn ký tự lạ → trả về danh sách rỗng (tầng gọi nên hiển
/// thị toàn bộ mục thay vì gọi hàm này).
pub async fn tim_tu_dien(
    pool: &SqlitePool,
    q: &str,
    limit: i64,
) -> Result<Vec<MucTuDien>, sqlx::Error> {
    let norm = chuan_hoa_tim_kiem(q);
    if norm.is_empty() {
        return Ok(Vec::new());
    }
    // Escape ký tự đặc biệt của cú pháp FTS5 bằng cách chỉ giữ token chữ/số —
    // `chuan_hoa_tim_kiem` đã làm việc đó, nên ở đây chỉ cần ghép tiền tố.
    let match_expr = norm
        .split_whitespace()
        .map(|t| format!("{t}*"))
        .collect::<Vec<_>>()
        .join(" AND ");

    let rows = sqlx::query(
        "SELECT d.* FROM tu_dien_fts f
         JOIN tu_dien d ON d.slug = f.slug
         WHERE tu_dien_fts MATCH ?
         ORDER BY bm25(tu_dien_fts, 0.0, 10.0, 5.0, 1.0)
         LIMIT ?",
    )
    .bind(&match_expr)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows.iter().map(map_muc).collect())
}

// ===========================================================================
// Hồ sơ người đã xem (giai đoạn 6)
// ===========================================================================

/// Một hồ sơ đã lưu, kèm lá số đã tính sẵn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoSo {
    pub id: i64,
    pub ten: String,
    /// 'YYYY-MM-DD'.
    pub solar_date: String,
    pub hour: u8,
    pub minute: u8,
    /// `nam` | `nu`.
    pub gender: String,
    pub ghi_chu: String,
    /// JSON của `TuViChart`.
    pub tuvi_json: String,
    /// JSON của `BatTuChart`.
    pub bat_tu_json: String,
    /// Phiên bản engine đã sinh ra hai cột JSON trên.
    pub engine_ver: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Dữ liệu để tạo hồ sơ mới.
#[derive(Debug, Clone)]
pub struct HoSoMoi<'a> {
    pub ten: &'a str,
    pub solar_date: &'a str,
    pub hour: u8,
    pub minute: u8,
    pub gender: &'a str,
    pub ghi_chu: &'a str,
    pub tuvi_json: &'a str,
    pub bat_tu_json: &'a str,
    pub engine_ver: &'a str,
}

fn map_ho_so(r: &sqlx::sqlite::SqliteRow) -> HoSo {
    HoSo {
        id: r.get("id"),
        ten: r.get("ten"),
        solar_date: r.get("solar_date"),
        hour: r.get::<i64, _>("hour") as u8,
        minute: r.get::<i64, _>("minute") as u8,
        gender: r.get("gender"),
        ghi_chu: r.get("ghi_chu"),
        tuvi_json: r.get("tuvi_json"),
        bat_tu_json: r.get("bat_tu_json"),
        engine_ver: r.get("engine_ver"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }
}

/// Tạo hồ sơ mới, trả về `id` vừa sinh.
pub async fn them_ho_so(pool: &SqlitePool, m: HoSoMoi<'_>) -> Result<i64, sqlx::Error> {
    let res = sqlx::query(
        "INSERT INTO ho_so
           (ten, solar_date, hour, minute, gender, ghi_chu, tuvi_json, bat_tu_json, engine_ver)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(m.ten)
    .bind(m.solar_date)
    .bind(m.hour as i64)
    .bind(m.minute as i64)
    .bind(m.gender)
    .bind(m.ghi_chu)
    .bind(m.tuvi_json)
    .bind(m.bat_tu_json)
    .bind(m.engine_ver)
    .execute(pool)
    .await?;
    Ok(res.last_insert_rowid())
}

/// Danh sách hồ sơ, mới nhất trước.
pub async fn danh_sach_ho_so(pool: &SqlitePool) -> Result<Vec<HoSo>, sqlx::Error> {
    let rows = sqlx::query("SELECT * FROM ho_so ORDER BY created_at DESC, id DESC")
        .fetch_all(pool)
        .await?;
    Ok(rows.iter().map(map_ho_so).collect())
}

/// Một hồ sơ theo `id`.
pub async fn lay_ho_so(pool: &SqlitePool, id: i64) -> Result<Option<HoSo>, sqlx::Error> {
    let row = sqlx::query("SELECT * FROM ho_so WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.as_ref().map(map_ho_so))
}

/// Sửa **tên và ghi chú** của một hồ sơ (không đụng tới lá số đã lưu).
///
/// Cố ý không cho sửa ngày giờ sinh: đổi ngày sinh nghĩa là một người khác /
/// một lá số khác, nên nên tạo hồ sơ mới thay vì sửa tại chỗ — sửa mà không tính
/// lại lá số sẽ để lại bản ghi mâu thuẫn.
///
/// Trả `false` nếu không có hồ sơ nào mang `id` đó.
pub async fn sua_ho_so(
    pool: &SqlitePool,
    id: i64,
    ten: Option<&str>,
    ghi_chu: Option<&str>,
) -> Result<bool, sqlx::Error> {
    if ten.is_none() && ghi_chu.is_none() {
        // Không có gì để sửa — vẫn báo có tồn tại hay không cho nhất quán.
        return Ok(lay_ho_so(pool, id).await?.is_some());
    }
    let res = sqlx::query(
        "UPDATE ho_so
            SET ten     = COALESCE(?, ten),
                ghi_chu = COALESCE(?, ghi_chu),
                updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now')
          WHERE id = ?",
    )
    .bind(ten)
    .bind(ghi_chu)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() > 0)
}

/// Xoá hồ sơ. Trả `false` nếu `id` không tồn tại.
pub async fn xoa_ho_so(pool: &SqlitePool, id: i64) -> Result<bool, sqlx::Error> {
    let res = sqlx::query("DELETE FROM ho_so WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(res.rows_affected() > 0)
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

    fn muc(slug: &str, title: &str, kind: &str, aliases: &str, body: &str) -> MucTuDien {
        MucTuDien {
            slug: slug.into(),
            title: title.into(),
            kind: kind.into(),
            nhom: "".into(),
            nguhanh: "".into(),
            amduong: "".into(),
            aliases: aliases.into(),
            body: body.into(),
            ord: 0,
        }
    }

    /// `đ` phải map thành `d` — đây là lý do tự chuẩn hoá thay vì dùng
    /// `remove_diacritics 2` của FTS5.
    #[test]
    fn chuan_hoa_bo_dau_va_map_chu_d() {
        assert_eq!(chuan_hoa_tim_kiem("Tử Vi"), "tu vi");
        assert_eq!(chuan_hoa_tim_kiem("Đào Hoa"), "dao hoa");
        assert_eq!(chuan_hoa_tim_kiem("Thiên Việt"), "thien viet");
        assert_eq!(chuan_hoa_tim_kiem("Cự Môn"), "cu mon");
        assert_eq!(chuan_hoa_tim_kiem("  Phá   Quân  "), "pha quan");
        // Ký tự lạ thành khoảng trắng, không sinh token rác.
        assert_eq!(chuan_hoa_tim_kiem("Tử-Vi (Đế tinh)"), "tu vi de tinh");
        assert_eq!(chuan_hoa_tim_kiem("!!!"), "");
    }

    /// Tìm kiếm phải hoạt động cả khi gõ có dấu lẫn không dấu, và khớp tiền tố.
    #[tokio::test]
    async fn tim_tu_dien_khong_dau_va_tien_to() {
        let pool = init_and_migrate("sqlite::memory:").await.unwrap();
        nap_tu_dien(
            &pool,
            &[
                muc("sao-tu-vi", "Tử Vi", "chinh-tinh", "Đế tinh", "Đế tinh đứng đầu Bắc Đẩu"),
                muc("sao-dao-hoa", "Đào Hoa", "phu-tinh", "Hàm Trì", "Chủ về duyên dáng"),
                muc("cung-menh", "Mệnh", "cung", "Mệnh viên", "Cung quan trọng nhất"),
            ],
        )
        .await
        .unwrap();

        let one = |v: Vec<MucTuDien>| v.into_iter().map(|m| m.slug).collect::<Vec<_>>();

        // Không dấu.
        assert_eq!(one(tim_tu_dien(&pool, "tu vi", 10).await.unwrap()), ["sao-tu-vi"]);
        // Có dấu.
        assert_eq!(one(tim_tu_dien(&pool, "Tử Vi", 10).await.unwrap()), ["sao-tu-vi"]);
        // `đ` → `d`: đây là ca mà tokenizer mặc định của FTS5 sẽ trượt.
        assert_eq!(one(tim_tu_dien(&pool, "dao hoa", 10).await.unwrap()), ["sao-dao-hoa"]);
        // Tiền tố.
        assert_eq!(one(tim_tu_dien(&pool, "menh", 10).await.unwrap()), ["cung-menh"]);
        // Tìm theo tên gọi khác.
        assert_eq!(one(tim_tu_dien(&pool, "ham tri", 10).await.unwrap()), ["sao-dao-hoa"]);
        // Không khớp gì.
        assert!(tim_tu_dien(&pool, "khongcogi", 10).await.unwrap().is_empty());
        // Query rỗng / rác → rỗng, không lỗi cú pháp FTS5.
        assert!(tim_tu_dien(&pool, "", 10).await.unwrap().is_empty());
        assert!(tim_tu_dien(&pool, "  !!  ", 10).await.unwrap().is_empty());
    }

    /// Ký tự đặc biệt của cú pháp FTS5 không được làm query nổ.
    #[tokio::test]
    async fn tim_tu_dien_khong_vo_voi_ky_tu_dac_biet() {
        let pool = init_and_migrate("sqlite::memory:").await.unwrap();
        nap_tu_dien(&pool, &[muc("sao-tu-vi", "Tử Vi", "chinh-tinh", "", "abc")])
            .await
            .unwrap();
        for q in ["\"", "*", "AND", "a OR b", "NEAR(", "tu*\"", "^", "-"] {
            let r = tim_tu_dien(&pool, q, 10).await;
            assert!(r.is_ok(), "query {q:?} làm vỡ FTS5: {:?}", r.err());
        }
    }

    /// Nạp lại phải xoá sạch bản cũ, không để mục đã gỡ còn sót.
    #[tokio::test]
    async fn nap_lai_tu_dien_xoa_muc_cu() {
        let pool = init_and_migrate("sqlite::memory:").await.unwrap();
        nap_tu_dien(&pool, &[muc("a", "Alpha", "cung", "", "x"), muc("b", "Beta", "cung", "", "y")])
            .await
            .unwrap();
        assert_eq!(tat_ca_tu_dien(&pool).await.unwrap().len(), 2);

        nap_tu_dien(&pool, &[muc("a", "Alpha 2", "cung", "", "x")]).await.unwrap();
        let all = tat_ca_tu_dien(&pool).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "Alpha 2");
        assert!(lay_tu_dien(&pool, "b").await.unwrap().is_none());
        // Chỉ mục FTS cũng phải sạch theo.
        assert!(tim_tu_dien(&pool, "beta", 10).await.unwrap().is_empty());
    }

    /// CRUD hồ sơ đầy đủ.
    #[tokio::test]
    async fn ho_so_crud() {
        let pool = init_and_migrate("sqlite::memory:").await.unwrap();
        assert!(danh_sach_ho_so(&pool).await.unwrap().is_empty());

        let id = them_ho_so(
            &pool,
            HoSoMoi {
                ten: "Nguyễn Văn A",
                solar_date: "1991-10-24",
                hour: 7,
                minute: 30,
                gender: "nam",
                ghi_chu: "ghi chú ban đầu",
                tuvi_json: r#"{"a":1}"#,
                bat_tu_json: r#"{"b":2}"#,
                engine_ver: "0.1.0",
            },
        )
        .await
        .unwrap();
        assert!(id > 0);

        let h = lay_ho_so(&pool, id).await.unwrap().unwrap();
        assert_eq!(h.ten, "Nguyễn Văn A");
        assert_eq!(h.hour, 7);
        assert_eq!(h.minute, 30);
        assert_eq!(h.gender, "nam");
        assert_eq!(h.tuvi_json, r#"{"a":1}"#);
        assert_eq!(h.engine_ver, "0.1.0");

        // Sửa chỉ ghi chú — tên giữ nguyên.
        assert!(sua_ho_so(&pool, id, None, Some("ghi chú mới")).await.unwrap());
        let h = lay_ho_so(&pool, id).await.unwrap().unwrap();
        assert_eq!(h.ghi_chu, "ghi chú mới");
        assert_eq!(h.ten, "Nguyễn Văn A");
        // Lá số KHÔNG bị đụng tới khi sửa ghi chú.
        assert_eq!(h.tuvi_json, r#"{"a":1}"#);

        // Sửa tên.
        assert!(sua_ho_so(&pool, id, Some("Trần Thị B"), None).await.unwrap());
        assert_eq!(lay_ho_so(&pool, id).await.unwrap().unwrap().ten, "Trần Thị B");

        assert_eq!(danh_sach_ho_so(&pool).await.unwrap().len(), 1);

        // id không tồn tại.
        assert!(!sua_ho_so(&pool, 9999, Some("x"), None).await.unwrap());
        assert!(!xoa_ho_so(&pool, 9999).await.unwrap());
        assert!(lay_ho_so(&pool, 9999).await.unwrap().is_none());

        assert!(xoa_ho_so(&pool, id).await.unwrap());
        assert!(danh_sach_ho_so(&pool).await.unwrap().is_empty());
    }

    /// Ràng buộc CHECK của schema phải chặn dữ liệu vô lý.
    #[tokio::test]
    async fn ho_so_chan_du_lieu_sai() {
        let pool = init_and_migrate("sqlite::memory:").await.unwrap();
        let mk = |gender: &'static str, hour: u8| async move {
            let pool = init_and_migrate("sqlite::memory:").await.unwrap();
            them_ho_so(
                &pool,
                HoSoMoi {
                    ten: "X", solar_date: "2000-01-01", hour, minute: 0,
                    gender, ghi_chu: "", tuvi_json: "{}", bat_tu_json: "{}",
                    engine_ver: "0.1.0",
                },
            )
            .await
        };
        assert!(mk("nam", 7).await.is_ok());
        assert!(mk("khac", 7).await.is_err(), "gender lạ phải bị CHECK chặn");
        assert!(mk("nu", 25).await.is_err(), "giờ 25 phải bị CHECK chặn");
        let _ = pool;
    }
}
