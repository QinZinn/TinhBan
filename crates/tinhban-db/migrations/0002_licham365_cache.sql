-- 0002_licham365_cache.sql
-- Cache kết quả scrape licham365.vn cho tính năng "ngày tốt/xấu" (giai đoạn 5).
--
-- Vì sao cache theo NGÀY DƯƠNG LỊCH: nội dung diễn giải của một ngày cụ thể là
-- tĩnh — licham365 không đổi bài viết cho ngày 15/3/2024 theo thời gian. Nên
-- bản ghi `ok` được coi là **vĩnh viễn**, không TTL. Chỉ bản ghi `error` mới có
-- TTL (do tầng ứng dụng quyết định) để một lần site sập không đầu độc cache
-- mãi mãi.
--
-- Bản ghi `error` được lưu lại (chứ không bỏ qua) có chủ đích: nó cho biết đã
-- thử và thất bại vì lý do gì, giúp debug khi site đổi cấu trúc HTML.
CREATE TABLE IF NOT EXISTS licham365_cache (
    -- Ngày dương lịch dạng 'YYYY-MM-DD' (khoá chính, 1 bản ghi / ngày).
    solar_date  TEXT PRIMARY KEY NOT NULL,
    -- 'ok' hoặc 'error'.
    status      TEXT NOT NULL CHECK (status IN ('ok', 'error')),
    -- JSON các mục diễn giải đã bóc được. NULL khi status='error'.
    payload     TEXT,
    -- Mô tả lỗi khi status='error' (timeout / HTTP 5xx / không khớp selector…).
    error       TEXT,
    -- URL đã gọi — ghi lại để sau này đổi URL pattern vẫn truy được nguồn cũ.
    source_url  TEXT NOT NULL,
    -- Thời điểm scrape (ISO-8601 UTC).
    fetched_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Truy vấn dọn dẹp hay dùng: "các bản ghi lỗi cũ hơn X" → lọc theo status +
-- fetched_at.
CREATE INDEX IF NOT EXISTS idx_licham365_cache_status_fetched
    ON licham365_cache (status, fetched_at);
