-- 0003_tu_dien.sql
-- Từ điển tử vi (giai đoạn 6): bảng nội dung + chỉ mục toàn văn FTS5.
--
-- Nội dung gốc là các file markdown trong `content/tu-dien/`, được nạp lại toàn
-- bộ mỗi lần app khởi động (39 mục — rẻ, và tránh sót bản cũ sau khi sửa file).
-- Vì vậy bảng này là **cache có thể dựng lại**, không phải nguồn sự thật.

CREATE TABLE IF NOT EXISTS tu_dien (
    -- Khoá tra cứu + URL, khớp `Sao::slug()` / `PalaceName::slug()`.
    slug      TEXT PRIMARY KEY NOT NULL,
    title     TEXT NOT NULL,
    -- 'chinh-tinh' | 'phu-tinh' | 'cung'
    kind      TEXT NOT NULL,
    -- Bắc Đẩu / Nam Đẩu / Trung Thiên / Lục Cát / Lục Sát / 12 cung…
    nhom      TEXT NOT NULL DEFAULT '',
    nguhanh   TEXT NOT NULL DEFAULT '',
    amduong   TEXT NOT NULL DEFAULT '',
    aliases   TEXT NOT NULL DEFAULT '',
    -- Thân bài markdown (chưa render).
    body      TEXT NOT NULL,
    -- Thứ tự hiển thị ổn định trong cùng một `kind`.
    ord       INTEGER NOT NULL DEFAULT 0
);

-- Chỉ mục toàn văn.
--
-- Vì sao chỉ index bản ĐÃ CHUẨN HOÁ (bỏ dấu, đ→d, thường hoá) thay vì text gốc:
-- tokenizer `unicode61 remove_diacritics 2` của SQLite bỏ được dấu thanh tiếng
-- Việt, nhưng KHÔNG map `đ` → `d` (đ là chữ cái riêng, không phải d + dấu).
-- Hệ quả: gõ "dao hoa" sẽ không ra "Đào Hoa". Tự chuẩn hoá ở tầng Rust rồi mới
-- index thì tra cứu không dấu hoạt động đều cho mọi chữ.
--
-- `slug` để UNINDEXED: chỉ cần lấy ra để JOIN, không cần tìm kiếm trong đó.
CREATE VIRTUAL TABLE IF NOT EXISTS tu_dien_fts USING fts5(
    slug UNINDEXED,
    title_norm,
    aliases_norm,
    body_norm,
    tokenize = 'unicode61'
);
