-- 0004_ho_so.sql
-- Hồ sơ người đã xem (giai đoạn 6). Thiết kế đã chốt: 1 người / 1 lần xem =
-- 1 bản ghi; không lưu lịch sử nhiều lần xem cho cùng một người.

CREATE TABLE IF NOT EXISTS ho_so (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    ten         TEXT NOT NULL,
    -- Ngày sinh Dương lịch 'YYYY-MM-DD'.
    solar_date  TEXT NOT NULL,
    -- Giờ/phút sinh (0..23, 0..59).
    hour        INTEGER NOT NULL CHECK (hour BETWEEN 0 AND 23),
    minute      INTEGER NOT NULL CHECK (minute BETWEEN 0 AND 59),
    -- 'nam' | 'nu'
    gender      TEXT NOT NULL CHECK (gender IN ('nam', 'nu')),
    ghi_chu     TEXT NOT NULL DEFAULT '',

    -- Lá số đã tính, serialize JSON. Lưu lại để hiển thị hồ sơ cũ mà không phải
    -- tính lại — cũng để giữ nguyên "lá số đã từng thấy" kể cả khi engine đổi.
    tuvi_json   TEXT NOT NULL,
    bat_tu_json TEXT NOT NULL,

    -- Phiên bản engine đã sinh ra hai cột JSON trên.
    --
    -- Có cột này vì lịch sử dự án: Bug #7 (hằng số epoch) từng làm sai trụ
    -- Năm/Tháng Bát Tự cho ngày sinh rơi đúng mốc tiết khí. Nếu về sau lại phát
    -- hiện lỗi tương tự, cột này cho biết hồ sơ nào được tính bằng bản nào để
    -- lọc ra mà tính lại, thay vì phải đoán.
    engine_ver  TEXT NOT NULL,

    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Danh sách hồ sơ luôn sắp theo thời điểm tạo, mới nhất trước.
CREATE INDEX IF NOT EXISTS idx_ho_so_created ON ho_so (created_at DESC);
