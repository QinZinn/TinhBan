-- 0001_app_meta.sql
-- Migration đầu tiên: bảng key-value đơn giản chỉ để xác nhận pipeline
-- `sqlx::migrate!` hoạt động. Bảng thật (hồ sơ người được xem, lá số Tử Vi, Bát
-- Tự, cache ngày tốt/xấu...) sẽ được thêm ở các migration sau (giai đoạn sau).
CREATE TABLE IF NOT EXISTS app_meta (
    key        TEXT PRIMARY KEY NOT NULL,
    value      TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);