//! Đối chiếu engine Trạch Nhật với **licham365.vn** trên 52 ngày mẫu.
//!
//! # Vì sao có danh sách "lệch đã biết"
//!
//! Một vài ngày ta **cố ý** khác licham365. Mỗi ngày như vậy phải được khai báo
//! trong [`LECH_DA_BIET`] kèm lý do. Test kiểm tra hai chiều:
//!
//!  1. ngày KHÔNG có trong danh sách mà lệch → **fail** (bắt regression thật);
//!  2. ngày CÓ trong danh sách mà lại khớp → cũng **fail** (buộc phải xoá mục đã
//!     lỗi thời, để danh sách không âm thầm mục ruỗng thành cái cớ bỏ qua lỗi).
//!
//! Không có cơ chế "bỏ qua mềm" nào khác.

use chrono::NaiveDate;
use tinhban_core::{danh_gia_ngay, KiengKy};

#[path = "trach_nhat_ref_data.rs"]
mod ref_data;
use ref_data::{RefDay, REF_DAYS};

/// Các trường có thể lệch, dùng làm khoá cho [`LECH_DA_BIET`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Truong {
    NgayAm,
    Tiet,
    Truc,
    HoangDao,
    KiengKy,
}

/// Nguyên nhân gốc của một khác biệt so với licham365.
///
/// Mọi khác biệt đều phải quy được về một trong số ít nguyên nhân dưới đây.
/// Test [`moi_khac_biet_deu_quy_ve_nguyen_nhan_da_biet`] chốt rằng số nguyên
/// nhân không phình ra — nếu xuất hiện kiểu lệch mới, phải hiểu và đặt tên cho
/// nó, chứ không được nhét thêm ngoại lệ rời rạc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NguyenNhan {
    /// Sóc (new moon) rơi sát nửa đêm giờ VN → lệch 1 ngày Âm lịch.
    SocSatNuaDem,
    /// Quy ước đặt tên tiết khí khác nhau (đầu ngày vs cuối ngày).
    QuyUocTenTiet,
    /// licham365 đặt mốc giao tiết sớm 1 ngày khi khoảnh khắc giao rơi vào
    /// rạng sáng.
    LichamSomMocTiet,
    /// Kết luận của licham365 mâu thuẫn với chính danh sách sao của nó.
    LichamTuMauThuan,
}

impl NguyenNhan {
    fn mo_ta(self) -> &'static str {
        match self {
            Self::SocSatNuaDem =>
                "Sóc rơi sát nửa đêm giờ VN nên đa thức new_moon() của Hồ Ngọc \
                 Đức (sai số vài phút) xếp mùng 1 lệch 1 ngày. Giới hạn đã ghi \
                 nhận từ giai đoạn 2. Hiếm: đo 602 ngày trên 20 tháng khác thì \
                 khớp 602/602; chỉ tháng 7/2026 sai (nhật thực toàn phần \
                 12/8/2026 lúc 17:46 UTC = 00:46 ngày 13/8 giờ VN). Kéo theo sai \
                 cả Tam Nương / Nguyệt Kỵ vì hai mục này đếm theo ngày Âm.",
            Self::QuyUocTenTiet =>
                "Ta gọi tên tiết theo trạng thái CUỐI ngày (ngày chứa khoảnh \
                 khắc giao tiết đã mang tên tiết mới); licham365 gọi theo trạng \
                 thái ĐẦU ngày. Hai quy ước lệch nhau đúng 1 ngày tại mỗi mốc. \
                 Chỉ ảnh hưởng NHÃN hiển thị, không ảnh hưởng tháng tiết khí \
                 dùng cho Trực.",
            Self::LichamSomMocTiet =>
                "licham365 đặt mốc giao tiết sớm hơn ta 1 ngày khi khoảnh khắc \
                 giao rơi vào rạng sáng. Đo trên 318 ngày: chỉ 4 ngày lệch, và \
                 cả 4 đều có mốc tiết thật rơi trong khoảng 01:57–05:16 giờ VN \
                 của ngày kế tiếp (Hàn Lộ 2024 01:57, Lập Đông 2024 05:16, Tiểu \
                 Thử 2025 03:04, Lập Xuân 2026 02:54). Ở các mốc rơi muộn hơn \
                 trong ngày (Lập Xuân 2025 21:07, Kinh Trập 2025 15:03, Bạch Lộ \
                 2026 21:41) hai bên khớp nhau. Ta theo quy ước cổ điển: ngày \
                 giao tiết là ngày dương lịch CHỨA khoảnh khắc đó. Bằng chứng \
                 licham365 sai: trên chính những trang đó, dòng 'Tiết:' vẫn ghi \
                 tiết CŨ trong khi Trực đã nhảy sang tháng MỚI.",
            Self::LichamTuMauThuan =>
                "licham365 kết luận 'Hoàng đạo (tốt)' nhưng chính trang đó liệt \
                 kê sao 'Chu tước hắc đạo'. Cả 3 ca đều cùng dạng: ngày Chi Dậu \
                 trong tháng Âm lịch 4 hoặc 10 (Chi tháng Tỵ/Hợi). Ta theo vòng \
                 12 Thần: Chu Tước → Hắc Đạo.",
        }
    }
}

/// Ngày + trường mà ta biết trước là lệch với licham365, kèm nguyên nhân gốc.
///
/// Nếu nguồn đối chiếu sửa lỗi của họ, test sẽ đỏ và buộc ta xoá mục tương ứng.
const LECH_DA_BIET: &[(&str, Truong, NguyenNhan)] = &[
    // — Sóc sát nửa đêm: tháng 7/2026 lệch 1 ngày Âm
    ("2026-08-19", Truong::NgayAm, NguyenNhan::SocSatNuaDem),
    ("2026-08-19", Truong::KiengKy, NguyenNhan::SocSatNuaDem),
    // — Quy ước đặt tên tiết khí
    ("2025-02-03", Truong::Tiet, NguyenNhan::QuyUocTenTiet),
    ("2025-03-05", Truong::Tiet, NguyenNhan::QuyUocTenTiet),
    ("2016-08-22", Truong::Tiet, NguyenNhan::QuyUocTenTiet),
    // — licham365 đặt mốc giao tiết sớm 1 ngày
    ("2024-10-07", Truong::Truc, NguyenNhan::LichamSomMocTiet),
    ("2024-11-06", Truong::Truc, NguyenNhan::LichamSomMocTiet),
    ("2025-07-06", Truong::Truc, NguyenNhan::LichamSomMocTiet),
    ("2026-02-03", Truong::Truc, NguyenNhan::LichamSomMocTiet),
    // — licham365 tự mâu thuẫn với danh sách sao của chính nó
    ("2024-05-21", Truong::HoangDao, NguyenNhan::LichamTuMauThuan),
    ("2024-11-05", Truong::HoangDao, NguyenNhan::LichamTuMauThuan),
    ("2025-05-04", Truong::HoangDao, NguyenNhan::LichamTuMauThuan),
];

fn tra_lech(date: &str, truong: Truong) -> Option<NguyenNhan> {
    LECH_DA_BIET
        .iter()
        .find(|(d, t, _)| *d == date && *t == truong)
        .map(|(_, _, nn)| *nn)
}

fn ten_kieng_ky(k: KiengKy) -> &'static str {
    match k {
        KiengKy::TamNuong => "TamNuong",
        KiengKy::NguyetKy => "NguyetKy",
        KiengKy::SatChu => "SatChu",
    }
}

/// So sánh một trường; trả `Some(mô tả)` nếu có vấn đề cần báo lỗi.
fn kiem_tra(date: &str, truong: Truong, khop: bool, chi_tiet: String) -> Option<String> {
    match (tra_lech(date, truong), khop) {
        // Khớp và không nằm trong danh sách lệch → OK.
        (None, true) => None,
        // Lệch mà không khai báo → lỗi thật.
        (None, false) => Some(format!("{date} {truong:?}: LỆCH KHÔNG KHAI BÁO — {chi_tiet}")),
        // Khai báo lệch và đúng là lệch → OK.
        (Some(_), false) => None,
        // Khai báo lệch nhưng lại khớp → mục đã lỗi thời, phải xoá.
        (Some(nn), true) => Some(format!(
            "{date} {truong:?}: đã KHỚP nhưng vẫn còn trong LECH_DA_BIET — \
             hãy xoá mục này. Nguyên nhân cũ ({nn:?}): {}",
            nn.mo_ta()
        )),
    }
}

#[test]
fn doi_chieu_52_ngay_voi_licham365() {
    let mut loi: Vec<String> = Vec::new();

    for RefDay {
        date,
        lunar_day,
        lunar_month,
        day_can,
        day_chi,
        tiet,
        truc,
        hoang_dao,
        gio_hoang_dao,
        kieng_ky,
    } in REF_DAYS
    {
        let d = NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("ngày mẫu hợp lệ");
        let a = danh_gia_ngay(d).unwrap_or_else(|e| panic!("{date}: danh_gia_ngay lỗi: {e}"));

        // --- Can Chi của ngày: KHÔNG bao giờ được phép lệch (chu kỳ 60 thuần
        // số học, không dính thiên văn). Sai ở đây là sai nền móng → panic ngay.
        assert_eq!(
            a.day_can_chi.stem.index(),
            *day_can,
            "{date}: Can của ngày sai"
        );
        assert_eq!(
            a.day_can_chi.branch.index(),
            *day_chi,
            "{date}: Chi của ngày sai"
        );

        // --- Ngày Âm lịch
        let am_khop = a.lunar_date.day == *lunar_day && a.lunar_date.month == *lunar_month;
        loi.extend(kiem_tra(
            date,
            Truong::NgayAm,
            am_khop,
            format!(
                "licham365 {}/{} vs ta {}/{}",
                lunar_day, lunar_month, a.lunar_date.day, a.lunar_date.month
            ),
        ));

        // --- Tiết khí
        loi.extend(kiem_tra(
            date,
            Truong::Tiet,
            a.tiet_khi == *tiet,
            format!("licham365 {:?} vs ta {:?}", tiet, a.tiet_khi),
        ));

        // --- Trực
        loi.extend(kiem_tra(
            date,
            Truong::Truc,
            a.truc.index() == *truc,
            format!(
                "licham365 idx {} vs ta {} ({})",
                truc,
                a.truc.index(),
                a.truc.name_vn()
            ),
        ));

        // --- Ngày Hoàng Đạo / Hắc Đạo (bỏ qua khi licham365 ghi "Bình thường")
        if *hoang_dao != 2 {
            let mong_doi = *hoang_dao == 1;
            loi.extend(kiem_tra(
                date,
                Truong::HoangDao,
                a.hoang_dao_hac_dao.is_hoang_dao == mong_doi,
                format!(
                    "licham365 {} vs ta {} ({})",
                    if mong_doi { "Hoàng Đạo" } else { "Hắc Đạo" },
                    a.hoang_dao_hac_dao.nhan_vn(),
                    a.hoang_dao_hac_dao.than.name_vn()
                ),
            ));
        }

        // --- Giờ Hoàng Đạo: cũng thuần số học theo Chi ngày → không cho lệch.
        let mut ta_gio: Vec<u8> = a.gio_hoang_dao.iter().map(|g| g.branch.index()).collect();
        ta_gio.sort_unstable();
        assert_eq!(
            ta_gio, *gio_hoang_dao,
            "{date}: giờ Hoàng Đạo sai (Chi ngày {})",
            a.day_can_chi.branch.name_vn()
        );

        // --- Kiêng kỵ
        let mut ta_kk: Vec<&str> = a.kieng_ky.iter().map(|k| ten_kieng_ky(*k)).collect();
        ta_kk.sort_unstable();
        let mut mong_kk: Vec<&str> = kieng_ky.to_vec();
        mong_kk.sort_unstable();
        loi.extend(kiem_tra(
            date,
            Truong::KiengKy,
            ta_kk == mong_kk,
            format!("licham365 {:?} vs ta {:?}", mong_kk, ta_kk),
        ));
    }

    assert!(
        loi.is_empty(),
        "{} vấn đề khi đối chiếu {} ngày mẫu:\n  - {}",
        loi.len(),
        REF_DAYS.len(),
        loi.join("\n  - ")
    );
}

/// Chốt chặn: mọi khác biệt phải quy về một tập **nhỏ** các nguyên nhân đã
/// hiểu rõ. Nếu ai đó gặp kiểu lệch mới, họ buộc phải phân tích và đặt tên cho
/// nó thay vì thêm ngoại lệ rời rạc để test xanh trở lại.
#[test]
fn moi_khac_biet_deu_quy_ve_nguyen_nhan_da_biet() {
    let mut nn: Vec<String> = LECH_DA_BIET
        .iter()
        .map(|(_, _, n)| format!("{n:?}"))
        .collect();
    nn.sort();
    nn.dedup();
    assert!(
        nn.len() <= 4,
        "có {} nguyên nhân lệch khác nhau ({nn:?}) — vượt 4 nguyên nhân đã \\
         phân tích; hãy điều tra kiểu lệch mới thay vì nới ngưỡng",
        nn.len()
    );
    // Mỗi nguyên nhân phải có mô tả không rỗng.
    for (_, _, n) in LECH_DA_BIET {
        assert!(!n.mo_ta().is_empty(), "{n:?} thiếu mô tả");
    }
}
