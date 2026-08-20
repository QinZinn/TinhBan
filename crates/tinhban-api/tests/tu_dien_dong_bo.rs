//! Từ điển phải phủ **đúng** những gì engine đã implement — không thiếu, không thừa.
//!
//! Đây là chốt chặn hai chiều:
//!  - thêm sao mới vào `Sao` mà quên viết mục từ điển → đỏ;
//!  - viết mục từ điển cho sao chưa implement → cũng đỏ.
//!
//! Chạy hoàn toàn offline: đọc bản markdown nhúng trong binary, không cần DB.

#[allow(dead_code)]
#[path = "../src/tu_dien.rs"]
mod tu_dien;

use std::collections::BTreeSet;
use tinhban_core::{PalaceName, Sao};

fn slug_trong_tu_dien() -> BTreeSet<String> {
    tu_dien::doc_tat_ca().into_iter().map(|m| m.slug).collect()
}

#[test]
fn tu_dien_phu_het_sao_va_cung_da_implement() {
    let co = slug_trong_tu_dien();

    let mut can_co: BTreeSet<String> = Sao::ALL.iter().map(|s| s.slug().to_string()).collect();
    can_co.extend(
        PalaceName::all_in_order()
            .iter()
            .map(|p| p.slug().to_string()),
    );

    let thieu: Vec<_> = can_co.difference(&co).cloned().collect();
    let thua: Vec<_> = co.difference(&can_co).cloned().collect();

    assert!(
        thieu.is_empty(),
        "THIẾU mục từ điển cho: {thieu:?} — engine có nhưng content/tu-dien/ chưa có file"
    );
    assert!(
        thua.is_empty(),
        "THỪA mục từ điển: {thua:?} — có file markdown nhưng engine chưa implement \
         (hoặc slug viết sai, không khớp Sao::slug() / PalaceName::slug())"
    );
}

/// Đúng 14 chính tinh, 13 phụ tinh, 12 cung.
///
/// Con số 13 phụ tinh là cố ý: đề bài giai đoạn 6 ghi "14 phụ tinh" nhưng liệt kê
/// ra 13 sao, và code cũng có 13. Test khoá theo code.
#[test]
fn dung_so_luong_tung_loai() {
    let muc = tu_dien::doc_tat_ca();
    let dem = |k: &str| muc.iter().filter(|m| m.kind == k).count();
    assert_eq!(dem("chinh-tinh"), 14, "phải có đúng 14 chính tinh");
    assert_eq!(dem("phu-tinh"), 13, "phải có đúng 13 phụ tinh");
    assert_eq!(dem("cung"), 12, "phải có đúng 12 cung");
    assert_eq!(muc.len(), 39);
}

/// Mọi mục phải có metadata dùng được cho UI, và `kind` phải nằm trong tập cho phép.
#[test]
fn metadata_hop_le() {
    for m in tu_dien::doc_tat_ca() {
        assert!(
            ["chinh-tinh", "phu-tinh", "cung"].contains(&m.kind.as_str()),
            "{}: kind {:?} không hợp lệ",
            m.slug,
            m.kind
        );
        assert!(!m.title.trim().is_empty(), "{}: thiếu title", m.slug);
        assert!(
            m.body.len() > 120,
            "{}: thân bài quá ngắn ({} ký tự) — có vẻ là mục rỗng",
            m.slug,
            m.body.len()
        );
        // Sao phải có Ngũ Hành + Âm Dương; cung thì không.
        if m.kind != "cung" {
            assert!(!m.nguhanh.is_empty(), "{}: sao thiếu `nguhanh`", m.slug);
            assert!(!m.amduong.is_empty(), "{}: sao thiếu `amduong`", m.slug);
            assert!(!m.nhom.is_empty(), "{}: sao thiếu `nhom`", m.slug);
        }
    }
}

/// Tên hiển thị trong từ điển phải khớp `name_vn()` của engine — tránh tình
/// trạng lá số hiện "Tử Vi" mà từ điển lại ghi tên khác.
#[test]
fn title_khop_ten_hien_thi_cua_engine() {
    let muc = tu_dien::doc_tat_ca();
    let tra = |slug: &str| {
        muc.iter()
            .find(|m| m.slug == slug)
            .map(|m| m.title.clone())
            .unwrap_or_default()
    };
    for s in Sao::ALL {
        assert_eq!(tra(s.slug()), s.name_vn(), "lệch tên ở {}", s.slug());
    }
    for p in PalaceName::all_in_order() {
        assert_eq!(tra(p.slug()), p.name_vn(), "lệch tên ở {}", p.slug());
    }
}

/// Nhóm Bắc Đẩu / Nam Đẩu / Trung Thiên của 14 chính tinh phải đúng cách chia
/// cổ điển (xem bảng "nguồn mâu thuẫn" trong `content/tu-dien/README.md`).
#[test]
fn phan_nhom_chinh_tinh_theo_cach_chia_co_dien() {
    let muc = tu_dien::doc_tat_ca();
    let nhom = |slug: &str| {
        muc.iter()
            .find(|m| m.slug == slug)
            .map(|m| m.nhom.clone())
            .unwrap_or_default()
    };
    for s in ["sao-tu-vi", "sao-vu-khuc", "sao-liem-trinh", "sao-tham-lang",
              "sao-cu-mon", "sao-pha-quan"] {
        assert_eq!(nhom(s), "Bắc Đẩu", "{s} phải thuộc Bắc Đẩu");
    }
    for s in ["sao-thien-phu", "sao-thien-co", "sao-thien-luong", "sao-thien-dong",
              "sao-thien-tuong", "sao-that-sat"] {
        assert_eq!(nhom(s), "Nam Đẩu", "{s} phải thuộc Nam Đẩu");
    }
    for s in ["sao-thai-duong", "sao-thai-am"] {
        assert_eq!(nhom(s), "Trung Thiên", "{s} phải thuộc Trung Thiên");
    }
}
