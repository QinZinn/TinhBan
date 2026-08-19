//! Kiểm tra bộ bóc HTML trên **một trang thật** của licham365.vn đã lưu sẵn.
//!
//! Fixture `fixtures/licham365-2024-03-15.html` là bản tải nguyên vẹn ngày
//! 15/3/2024. Test này chạy **offline** — không gọi mạng, nên CI/máy không mạng
//! vẫn xanh.
//!
//! Vai trò: khoá hành vi của `parse_sections` trên HTML thật, phức tạp hơn
//! nhiều so với HTML tối giản trong unit test. Nếu ai đó "dọn dẹp" selector hay
//! logic cắt tiêu đề mà làm hỏng, test này đỏ ngay.
//!
//! Lưu ý phạm vi: fixture là ảnh chụp tĩnh nên **không** phát hiện được việc
//! licham365 đổi cấu trúc trong tương lai. Việc đó chỉ lộ ra khi chạy thật, qua
//! log `ScrapeError::NoSections` / `TooFewSections`.

// `main.rs` là crate binary nên test tích hợp không `use` được module của nó.
// Cách gọn nhất: include thẳng file nguồn của bộ parser (nó không phụ thuộc
// phần còn lại của binary).
// `dead_code` được tắt có chủ đích: test này chỉ dùng phần parse, nên các hàm
// gọi mạng (`fetch_detail`, `build_client`) hiển nhiên không được đụng tới ở đây.
#[allow(dead_code)]
#[path = "../src/scrape/licham365.rs"]
mod licham365;

use licham365::parse_sections;

const FIXTURE: &str = include_str!("fixtures/licham365-2024-03-15.html");

#[test]
fn boc_duoc_toan_bo_muc_tu_trang_that() {
    let d = parse_sections(FIXTURE, "https://licham365.vn/lich-am-ngay-15-thang-3-nam-2024")
        .expect("trang thật phải bóc được");

    let titles: Vec<&str> = d.sections.iter().map(|s| s.title.as_str()).collect();

    // Các mục cốt lõi — đây là phần "diễn giải" mà giai đoạn 5 cần.
    for can_co in [
        "Giờ hoàng đạo & Giờ hắc đạo",
        "Ngũ Hành",
        "Ngày bách kỵ",
        "Theo Ngọc Hạp Thông Thư",
        "Xuất hành",
    ] {
        assert!(
            titles.contains(&can_co),
            "thiếu mục {can_co:?}; các mục bóc được: {titles:?}"
        );
    }

    // Mục Trực có tên động ("Trực: Bế") nên khớp theo tiền tố.
    assert!(
        titles.iter().any(|t| t.starts_with("Trực:")),
        "thiếu mục Trực; các mục: {titles:?}"
    );

    assert!(
        d.sections.len() >= 10,
        "trang thật phải có ≥10 mục, chỉ bóc được {}",
        d.sections.len()
    );
}

#[test]
fn noi_dung_muc_khong_lap_lai_tieu_de_va_khong_rong() {
    let d = parse_sections(FIXTURE, "u").unwrap();
    for s in &d.sections {
        assert!(!s.title.trim().is_empty(), "có mục tiêu đề rỗng");
        assert!(!s.body.trim().is_empty(), "mục {:?} có body rỗng", s.title);
        assert!(
            !s.body.starts_with(&s.title),
            "mục {:?} bị lặp tiêu đề trong body",
            s.title
        );
    }
}

#[test]
fn muc_truc_chua_nen_lam_va_khong_nen_lam() {
    let d = parse_sections(FIXTURE, "u").unwrap();
    let truc = d
        .sections
        .iter()
        .find(|s| s.title.starts_with("Trực:"))
        .expect("phải có mục Trực");
    assert_eq!(truc.title, "Trực: Bế", "ngày 15/3/2024 là Trực Bế");
    assert!(truc.body.contains("Nên làm"), "body: {}", truc.body);
    assert!(truc.body.contains("Không nên làm"), "body: {}", truc.body);
}

#[test]
fn boc_duoc_tom_tat_ngay_hoang_dao() {
    let d = parse_sections(FIXTURE, "u").unwrap();
    let t = d.tom_tat.expect("trang thật phải có đoạn tóm tắt");
    assert!(t.contains("Hoàng đạo"), "tóm tắt: {t}");
    assert!(t.contains("15 tháng 3 năm 2024"), "tóm tắt: {t}");
    assert!(t.len() < 400, "tóm tắt cắt quá dài: {} ký tự", t.len());
}
