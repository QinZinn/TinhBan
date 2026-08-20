//! Từ điển tử vi: nạp nội dung markdown → SQLite (FTS5) và phục vụ tra cứu.
//!
//! # Nội dung đến từ đâu
//!
//! Nguồn sự thật là các file markdown trong `content/tu-dien/` (xem README ở đó).
//! Chúng được **nhúng thẳng vào binary** lúc biên dịch, nên deploy chỉ cần copy
//! đúng một file — hợp với kiểu triển khai "1 binary + systemd" của dự án.
//!
//! Muốn sửa nội dung mà không build lại: đặt `TINHBAN_CONTENT_DIR` trỏ tới thư
//! mục markdown trên đĩa, app sẽ đọc từ đó thay cho bản nhúng.
//!
//! Mỗi lần khởi động, [`nap_vao_db`] xoá sạch bảng rồi ghi lại từ đầu — bảng
//! `tu_dien` là **cache dựng lại được**, không phải nơi lưu trữ gốc.

use include_dir::{include_dir, Dir};
use tinhban_db::MucTuDien;

/// Toàn bộ `content/tu-dien/` nhúng lúc biên dịch.
static CONTENT: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../content/tu-dien");

/// Biến môi trường trỏ tới thư mục markdown trên đĩa (ghi đè bản nhúng).
pub const ENV_CONTENT_DIR: &str = "TINHBAN_CONTENT_DIR";

/// Tách frontmatter đơn giản dạng `key: value` giữa hai dòng `---`.
///
/// Cố ý **không** dùng parser YAML đầy đủ: frontmatter ở đây chỉ có các cặp
/// khoá–giá trị một dòng, thêm một dependency YAML cho việc đó là thừa. Nếu sau
/// này cần cấu trúc lồng nhau thì mới đổi.
fn tach_frontmatter(raw: &str) -> Option<(Vec<(String, String)>, String)> {
    let rest = raw.strip_prefix("---")?.trim_start_matches(['\r', '\n']);
    let end = rest.find("\n---")?;
    let (fm, body) = rest.split_at(end);
    let body = body.trim_start_matches("\n---").trim_start_matches(['\r', '\n']);

    let mut kv = Vec::new();
    for line in fm.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            kv.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    Some((kv, body.to_string()))
}

fn lay<'a>(kv: &'a [(String, String)], key: &str) -> &'a str {
    kv.iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
        .unwrap_or("")
}

/// Chuyển nội dung một file markdown thành [`MucTuDien`].
///
/// Trả `None` (kèm lý do qua `tracing`) nếu thiếu frontmatter hoặc thiếu `slug` /
/// `title` — bỏ qua một file hỏng còn hơn làm sập cả app lúc khởi động.
fn phan_tich(ten_file: &str, raw: &str, ord: i64) -> Option<MucTuDien> {
    let Some((kv, body)) = tach_frontmatter(raw) else {
        tracing::warn!(file = ten_file, "bỏ qua: không có frontmatter `---`");
        return None;
    };
    let slug = lay(&kv, "slug").to_string();
    let title = lay(&kv, "title").to_string();
    if slug.is_empty() || title.is_empty() {
        tracing::warn!(file = ten_file, "bỏ qua: thiếu `slug` hoặc `title`");
        return None;
    }
    Some(MucTuDien {
        slug,
        title,
        kind: lay(&kv, "kind").to_string(),
        nhom: lay(&kv, "nhom").to_string(),
        nguhanh: lay(&kv, "nguhanh").to_string(),
        amduong: lay(&kv, "amduong").to_string(),
        aliases: lay(&kv, "aliases").to_string(),
        body,
        ord,
    })
}

/// Đọc toàn bộ mục từ điển: ưu tiên thư mục ngoài (`TINHBAN_CONTENT_DIR`), nếu
/// không có thì dùng bản nhúng trong binary.
///
/// `README.md` trong thư mục nội dung bị bỏ qua (không có frontmatter).
pub fn doc_tat_ca() -> Vec<MucTuDien> {
    let mut files: Vec<(String, String)> = Vec::new();

    match std::env::var(ENV_CONTENT_DIR) {
        Ok(dir) if !dir.trim().is_empty() => {
            tracing::info!(dir = %dir, "từ điển: đọc từ thư mục ngoài");
            match std::fs::read_dir(&dir) {
                Ok(entries) => {
                    for e in entries.flatten() {
                        let p = e.path();
                        if p.extension().and_then(|x| x.to_str()) != Some("md") {
                            continue;
                        }
                        let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
                        match std::fs::read_to_string(&p) {
                            Ok(c) => files.push((name, c)),
                            Err(e) => tracing::warn!(file = %name, error = %e, "không đọc được"),
                        }
                    }
                }
                Err(e) => {
                    // Thư mục chỉ định sai → cảnh báo rõ rồi quay về bản nhúng,
                    // thay vì im lặng phục vụ từ điển rỗng.
                    tracing::warn!(dir = %dir, error = %e,
                        "không mở được thư mục nội dung, dùng bản nhúng trong binary");
                }
            }
        }
        _ => {}
    }

    if files.is_empty() {
        for f in CONTENT.files() {
            let name = f.path().file_name().unwrap_or_default().to_string_lossy().to_string();
            if !name.ends_with(".md") {
                continue;
            }
            if let Some(c) = f.contents_utf8() {
                files.push((name, c.to_string()));
            }
        }
    }

    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
        .iter()
        .enumerate()
        .filter_map(|(i, (name, raw))| phan_tich(name, raw, i as i64))
        .collect()
}

/// Nạp từ điển vào DB (xoá sạch rồi ghi lại). Gọi một lần lúc khởi động.
pub async fn nap_vao_db(pool: &sqlx::SqlitePool) -> Result<usize, sqlx::Error> {
    let muc = doc_tat_ca();
    tinhban_db::nap_tu_dien(pool, &muc).await?;
    Ok(muc.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tach_frontmatter_co_ban() {
        let raw = "---\nslug: a\ntitle: A\n---\n\n# Thân bài\n\nnội dung\n";
        let (kv, body) = tach_frontmatter(raw).expect("phải tách được");
        assert_eq!(lay(&kv, "slug"), "a");
        assert_eq!(lay(&kv, "title"), "A");
        assert!(body.starts_with("# Thân bài"));
    }

    #[test]
    fn frontmatter_gia_tri_rong_van_hop_le() {
        // Các mục "cung" có `nguhanh:` và `amduong:` để trống.
        let raw = "---\nslug: cung-menh\ntitle: Mệnh\nnguhanh: \namduong:\n---\n\nx\n";
        let (kv, _) = tach_frontmatter(raw).unwrap();
        assert_eq!(lay(&kv, "nguhanh"), "");
        assert_eq!(lay(&kv, "amduong"), "");
        assert_eq!(lay(&kv, "title"), "Mệnh");
    }

    #[test]
    fn thieu_frontmatter_thi_bo_qua_chu_khong_panic() {
        assert!(tach_frontmatter("không có gì").is_none());
        assert!(phan_tich("x.md", "không có gì", 0).is_none());
        // Có frontmatter nhưng thiếu slug.
        assert!(phan_tich("x.md", "---\ntitle: A\n---\nbody", 0).is_none());
    }

    /// Bản nhúng phải đọc được và không rỗng — bắt lỗi đường dẫn `include_dir!`
    /// sai ngay lúc chạy test thay vì lúc deploy.
    #[test]
    fn ban_nhung_doc_duoc() {
        let muc = doc_tat_ca();
        assert!(
            muc.len() >= 39,
            "phải nạp được ít nhất 39 mục, chỉ thấy {}",
            muc.len()
        );
        assert!(muc.iter().all(|m| !m.slug.is_empty() && !m.title.is_empty()));
        assert!(muc.iter().all(|m| !m.body.trim().is_empty()), "có mục thân bài rỗng");
        // README.md không có frontmatter nên phải bị loại.
        assert!(!muc.iter().any(|m| m.title.contains("Từ điển Tử Vi — nội dung")));
    }
}
