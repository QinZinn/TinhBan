//! Tính ngày tiết khí (節氣) dựa trên kinh độ Mặt Trời, dùng cho Bát Tự.
//!
//! # Đặc điểm
//!
//! Bát Tự dùng 12 tiết (節, major solar terms) làm ranh giới tháng/năm:
//!
//! | Tiết       | Kinh độ MT | Tháng BT (branchmonth) |
//! |------------|-----------|------------------------|
//! | Lập Xuân   | 315°       | 1 — Dần (year start) |
//! | Kinh Trập  | 345°       | 2 — Mão                |
//! | Thanh Minh |  15°       | 3 — Thìn               |
//! | Lập Hạ     |  45°       | 4 — Tỵ                 |
//! | Mang Chủng |  75°       | 5 — Ngọ                |
//! | Tiểu Thử   | 105°       | 6 — Mùi                |
//! | Lập Thu    | 135°       | 7 — Thân               |
//! | Bạch Lộ    | 165°       | 8 — Dậu                |
//! | Hàn Lộ     | 195°       | 9 — Tuất               |
//! | Lập Đông   | 225°       | 10 — Hợi               |
//! | Đại Tuyết   | 255°       | 11 — Tý                |
//! | Tiểu Hàn   | 285°       | 12 — Sửu              |
//!
//! 12 tiết cách nhau 30° kinh độ Mặt Trời, lùi 15° so với các trung khí
//! (khí, e.g. Vũ Thủy 330°, Xuân Phân 0°, ...).
//!
//! # Thuật toán
//!
//! Tái sử dụng `sun_longitude_at_local_midnight` trong `astronomy.rs` (cùng
//! công thức trời văn của Hồ Ngọc Đức giai đoạn 2). Tìm JD của ngày có kinh độ
//! Mặt Trời tại local midnight (= 00:00 VN) chuyển từ dưới sang trên `target_deg`
//! trong chiều tiến của năm — ngày tiết khí là calendar day CHỨA khoảnh khắc
//! chuyển này, tức `JD transitioning = jd - 1` so với `jd` có L(morning) >= target.
//!
//! # Độ chính xác
//!
//! Giai đoạn 4 ghi nhận "±1 ngày, 4/6 mốc Lập Xuân lệch so với HK Observatory"
//! và quy cho giới hạn của công thức polynomial Hồ Ngọc Đức. Giai đoạn 5 truy
//! ra nguyên nhân thật: **bug hằng số epoch** trong
//! `sun_longitude_deg_at_local_midnight` (dùng `2451545.5` trong khi đã trừ 0.5
//! ngày riêng → double-count nửa ngày). Xem doc-comment của hàm đó trong
//! `astronomy.rs`.
//!
//! Sau khi sửa, đối chiếu lịch vạn niên:
//!  - **24/24** tiết khí năm 2024 khớp đúng ngày;
//!  - **10/10** mốc Lập Xuân 2017–2026 khớp đúng ngày.
//!
//! Vì vậy ranh giới tháng Bát Tự / tháng Trực nay đáng tin cả ở ngày sát biên.

use crate::astronomy::{jd_from_date, jd_to_date, sun_longitude_deg_at_local_midnight, VN_TZ};

/// 12 tiết khí theo thứ tự Lập Xuân → Tiểu Hàn, kèm kinh độ Mặt Trời (°) và
/// tên tiếng Việt. Index 0..11 ứng với tháng BT 1..12 (Dần..Sửu).
pub const TIET_KHI_TABLE: [(u32, &str); 12] = [
    (315, "Lập Xuân"),    // → tháng 1, Dần
    (345, "Kinh Trập"),   // → tháng 2, Mão
    (15, "Thanh Minh"),   // → tháng 3, Thìn
    (45, "Lập Hạ"),       // → tháng 4, Tỵ
    (75, "Mang Chủng"),   // → tháng 5, Ngọ
    (105, "Tiểu Thử"),    // → tháng 6, Mùi
    (135, "Lập Thu"),     // → tháng 7, Thân
    (165, "Bạch Lộ"),     // → tháng 8, Dậu
    (195, "Hàn Lộ"),      // → tháng 9, Tuất
    (225, "Lập Đông"),    // → tháng 10, Hợi
    (255, "Đại Tuyết"),   // → tháng 11, Tý
    (285, "Tiểu Hàn"),    // → tháng 12, Sửu
];

/// Tính JD của tiết (major solar term) tại kinh độ Mặt Trời `target_deg`, rơi
/// trong năm Dương lịch `search_year`. Trả về JD nguyên (Ho convention = trưa UTC
/// của ngày tiết khí), hoặc `None` nếu không tìm thấy.
///
/// Algorithm: duyệt tuần tự từng ngày trong `search_year`, tính kinh độ Mặt Trời
/// tại local midnight VN; phát hiện "transition" khi L_prev < target và L_curr
/// không nhỏ hơn target (trong chiều tiến của năm, đã xử lý wrap 0/360). Tiết
/// khí rơi vào calendar day (jd - 1) của JD detection — tức transition happens
/// DURING previous calendar day whose midnight had L < target and ends với
/// L không nhỏ hơn target.
pub fn find_tiet_khi_jd(target_deg: f64, search_year: i32) -> Option<i64> {
    let start_jd = jd_from_date(1, 1, search_year.into());
    let end_jd = jd_from_date(31, 12, search_year.into());
    let mut prev_deg = sun_longitude_deg_at_local_midnight(start_jd - 1, VN_TZ);
    for jd in start_jd..=end_jd {
        let curr_deg = sun_longitude_deg_at_local_midnight(jd, VN_TZ);
        // Forward angular distance prev→curr (deg, ~0.985 typical, wrap-aware)
        let delta_forward = ((curr_deg - prev_deg) % 360.0 + 360.0) % 360.0;
        if !(0.5..=5.0).contains(&delta_forward) {
            prev_deg = curr_deg;
            continue; // anomaly (year wrap) hoặc không phải ngày thường
        }
        // Forward angular distance prev→target
        let dist_prev_to_target = ((target_deg - prev_deg) % 360.0 + 360.0) % 360.0;
        if dist_prev_to_target <= delta_forward {
            return Some(jd - 1); // transition occurred during calendar day (jd - 1)
        }
        prev_deg = curr_deg;
    }
    None
}

/// Lấy JD của Lập Xuân (315°) trong năm Dương lịch `year`. Nếu Ho's formula
/// không tìm thấy trong `year`, fallback: Lập Xuân của year-1+1 (vì có hiếm year
/// Lập Xuân có thể dự đoán rơi quá 31/12 — gần như không xảy ra).
pub fn lap_xuan_jd(year: i32) -> i64 {
    find_tiet_khi_jd(315.0, year).unwrap_or_else(|| {
        // Ho's formula vẫn có khả năng lệch ±5 ngày so với ngày thực tế, không
        // quá 31/12 → tìm thấy. Nếu đến mức này, nó bug nội bộ — panic.
        panic!("Lập Xuân not found for year {} — Ho algorithm anomaly", year)
    })
}

/// Lấy 12 tiết khí JD của năm Bát Tự `bt_year` (từ Lập Xuân bt_year đến trước
/// Lập Xuân bt_year+1). Trả về mảng 12 phần tử theo thứ tự TIET_KHI_TABLE.
///
/// Lưu ý: tiết 11 (Đại Tuyết 255°) rơi vào khoảng 6-7/12 của `bt_year`; tiết 12
/// (Tiểu Hàn 285°) rơi vào khoảng 5-6/1 của `bt_year+1` → tìm trong năm Dương
/// lịch tiếp theo.
pub fn tiet_khi_jds_of_bt_year(bt_year: i32) -> [i64; 12] {
    let mut jds = [0i64; 12];
    for (i, &(deg, _name)) in TIET_KHI_TABLE.iter().enumerate() {
        // For tiết cuối (Tiểu Hàn 285°), JD falls trong calendar year (bt_year+1).
        let search_year = if deg == 285 {
            bt_year + 1
        } else {
            bt_year
        };
        jds[i] = find_tiet_khi_jd(deg as f64, search_year).unwrap_or_else(|| {
            panic!("Tiết khí index {} not found for BT year {}", i, bt_year)
        });
    }
    jds
}

/// 24 tiết khí (12 **tiết** 節 xen kẽ 12 **trung khí** 氣), theo thứ tự kinh độ
/// Mặt Trời bắt đầu từ Lập Xuân 315°, mỗi bước 15°. Index 0..23.
///
/// Index chẵn = tiết (節, mở đầu tháng Can Chi); index lẻ = trung khí (氣, giữa
/// tháng). `TIET_KHI_TABLE` ở trên chính là các index chẵn của bảng này.
pub const TIET_KHI_24: [(u32, &str); 24] = [
    (315, "Lập Xuân"),
    (330, "Vũ Thủy"),
    (345, "Kinh Trập"),
    (0, "Xuân Phân"),
    (15, "Thanh Minh"),
    (30, "Cốc Vũ"),
    (45, "Lập Hạ"),
    (60, "Tiểu Mãn"),
    (75, "Mang Chủng"),
    (90, "Hạ Chí"),
    (105, "Tiểu Thử"),
    (120, "Đại Thử"),
    (135, "Lập Thu"),
    (150, "Xử Thử"),
    (165, "Bạch Lộ"),
    (180, "Thu Phân"),
    (195, "Hàn Lộ"),
    (210, "Sương Giáng"),
    (225, "Lập Đông"),
    (240, "Tiểu Tuyết"),
    (255, "Đại Tuyết"),
    (270, "Đông Chí"),
    (285, "Tiểu Hàn"),
    (300, "Đại Hàn"),
];

/// Kinh độ Mặt Trời (độ, [0, 360)) tại **cuối** ngày Dương lịch có JD `jd`
/// — tức 00:00 VN của ngày kế tiếp.
///
/// Đây là đại lượng quyết định "ngày `jd` thuộc tiết nào": nếu khoảnh khắc
/// giao tiết xảy ra *trong* ngày `jd` thì ngày đó đã thuộc tiết mới, và giá trị
/// cuối ngày đã vượt qua mốc. Dùng chung một quy ước với `find_tiet_khi_jd`
/// (hàm đó trả `jd - 1` cho ngày mà `L(jd)` vừa vượt mốc), nên hai hàm luôn
/// nhất quán với nhau — đừng đổi một bên mà không đổi bên kia.
fn sun_longitude_at_end_of_day(jd: i64) -> f64 {
    sun_longitude_deg_at_local_midnight(jd + 1, VN_TZ)
}

/// Số thứ tự tháng Can Chi theo **tiết khí** (節月) chứa ngày có JD `jd`:
/// `1` = Dần (từ Lập Xuân), `2` = Mão (từ Kinh Trập), …, `12` = Sửu (từ Tiểu Hàn).
///
/// Đây là "tháng" mà **12 Trực** và **Bát Tự** dùng — KHÔNG phải tháng Âm lịch.
/// Hai loại tháng lệch nhau vài ngày mỗi tháng (mốc tiết khí không trùng mùng 1
/// âm lịch), và dùng nhầm tháng âm là lỗi phổ biến khi tự implement 12 Trực.
///
/// Chi phí: đúng **1** lần tính kinh độ Mặt Trời (khác `tiet_khi_jds_of_bt_year`
/// phải quét cả năm), nên gọi được cho từng ngày mà không lo hiệu năng.
pub fn tiet_month_index(jd: i64) -> u8 {
    let deg = sun_longitude_at_end_of_day(jd);
    // Các tiết (節) nằm ở 315°, 345°, 15°, … tức ≡ 15 (mod 30). Quy về gốc
    // Lập Xuân 315° rồi chia sector 30°.
    let from_lap_xuan = (deg - 315.0).rem_euclid(360.0);
    (from_lap_xuan / 30.0).floor() as u8 + 1
}

/// Địa Chi của tháng tiết khí chứa `jd`: tháng 1 → Dần (index 2), tháng 2 → Mão
/// (3), …, tháng 12 → Sửu (index 1). Trả **index 0..11** của `EarthlyBranch`.
pub fn tiet_month_branch_index(jd: i64) -> u8 {
    // Tháng 1 (Dần) = branch index 2 → (m - 1 + 2) mod 12.
    (tiet_month_index(jd) + 1) % 12
}

/// Tiết khí (trong 24) đang có hiệu lực tại ngày có JD `jd`. Trả
/// `(kinh_độ, tên)` — cùng phần tử của [`TIET_KHI_24`].
///
/// "Đang có hiệu lực" = tiết/trung khí gần nhất đã bắt đầu tính tới hết ngày
/// `jd`, giống cách lịch vạn niên hiển thị dòng "Tiết: …".
pub fn current_tiet_khi(jd: i64) -> (u32, &'static str) {
    let deg = sun_longitude_at_end_of_day(jd);
    let from_lap_xuan = (deg - 315.0).rem_euclid(360.0);
    let idx = (from_lap_xuan / 15.0).floor() as usize;
    TIET_KHI_24[idx.min(23)]
}

/// Trả JD-1 chuyển `current_jd` về (year, month, day) Dương lịch, dùng cho
/// debug / logging.
pub fn jd_to_ymd(jd: i64) -> (i32, u32, u32) {
    let (d, m, y) = jd_to_date(jd);
    (y as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lap_xuan_jd_in_early_february_for_every_year_1950_2100() {
        // Lập Xuân phải rơi vào khoảng 3-6/2 (mặc dù Ho có ±1-day precision so
        // với nguồn chính thức, daily range 3-5/2 begin năm Dương là ổn).
        for year in (1950..=2100).step_by(7) {
            let jd = lap_xuan_jd(year);
            let (y, m, d) = jd_to_ymd(jd);
            assert_eq!(m, 2, "year {}: tháng phải = 2, got {}", year, m);
            assert_eq!(y, year, "year {}: năm Dương lịch phải = {}", year, y);
            assert!(
                (3..=6).contains(&d),
                "year {}: Lập Xuân predicted day {} outside reasonable 3-6/2",
                year, d
            );
        }
    }

    #[test]
    fn tieu_han_jd_in_early_january_for_test_years() {
        for year in [2024i32, 2025, 1990, 2000] {
            let jd = find_tiet_khi_jd(285.0, year).expect("must find Tiểu Hàn");
            let (y, m, d) = jd_to_ymd(jd);
            assert_eq!(m, 1, "Tiểu Hàn year {}: must be Jan", year);
            assert_eq!(y, year);
            assert!((5..=7).contains(&d), "Tiểu Hàn year {}: day {} outside 5-7/1", year, d);
        }
    }

    #[test]
    fn all_12_tiet_khi_jds_are_strictly_increasing_for_any_year() {
        let jds_2024 = tiet_khi_jds_of_bt_year(2024);
        for w in jds_2024.windows(2) {
            assert!(w[1] > w[0], "tiết JDs not increasing: {:?}", w);
        }
        // 12 tiết cách nhau ~29-31 ngày
        for w in jds_2024.windows(2) {
            let diff = w[1] - w[0];
            assert!((28..=32).contains(&diff), "tiết diff {} days outside 28-32", diff);
        }
    }

    #[test]
    fn tiet_khi_first_jd_is_lap_xuan_of_that_calendar_year() {
        let jds = tiet_khi_jds_of_bt_year(2024);
        // jd[0] = Lập Xuân 2024 rơi trong calendar 2024 (voucher)
        let (y, m, _d) = jd_to_ymd(jds[0]);
        assert_eq!(y, 2024);
        assert_eq!(m, 2);
    }

    /// **Regression cho bug epoch (giai đoạn 5).** Trước khi sửa hằng số
    /// `2451545.5` → `2451545.0` trong `sun_longitude_deg_at_local_midnight`,
    /// 5/10 mốc Lập Xuân dưới đây lệch đúng 1 ngày.
    ///
    /// Ngày chuẩn lấy từ lịch vạn niên (giờ VN, UTC+7).
    #[test]
    fn lap_xuan_khop_lich_van_nien_2017_2026() {
        let chuan = [
            (2017, 3u32),
            (2018, 4),
            (2019, 4),
            (2020, 4),
            (2021, 3),
            (2022, 4),
            (2023, 4),
            (2024, 4),
            (2025, 3),
            (2026, 4),
        ];
        for (nam, ngay) in chuan {
            let (y, m, d) = jd_to_ymd(lap_xuan_jd(nam));
            assert_eq!(
                (y, m, d),
                (nam, 2, ngay),
                "Lập Xuân {nam} phải là {ngay}/2/{nam}"
            );
        }
    }

    /// Cả 24 tiết khí của năm 2024 phải khớp đúng NGÀY với lịch vạn niên.
    ///
    /// Đây là bài kiểm tra chặt nhất cho `sun_longitude_deg_at_local_midnight`:
    /// 24 mốc trải khắp vòng hoàng đạo, sai hằng số epoch dù nhỏ cũng làm lệch
    /// hàng loạt.
    #[test]
    fn du_24_tiet_khi_nam_2024_khop_lich_van_nien() {
        // (kinh độ, tên, ngày, tháng) — giờ VN.
        let chuan: [(f64, &str, u32, u32); 24] = [
            (285.0, "Tiểu Hàn", 6, 1),
            (300.0, "Đại Hàn", 20, 1),
            (315.0, "Lập Xuân", 4, 2),
            (330.0, "Vũ Thủy", 19, 2),
            (345.0, "Kinh Trập", 5, 3),
            (0.0, "Xuân Phân", 20, 3),
            (15.0, "Thanh Minh", 4, 4),
            (30.0, "Cốc Vũ", 19, 4),
            (45.0, "Lập Hạ", 5, 5),
            (60.0, "Tiểu Mãn", 20, 5),
            (75.0, "Mang Chủng", 5, 6),
            (90.0, "Hạ Chí", 21, 6),
            (105.0, "Tiểu Thử", 6, 7),
            (120.0, "Đại Thử", 22, 7),
            (135.0, "Lập Thu", 7, 8),
            (150.0, "Xử Thử", 22, 8),
            (165.0, "Bạch Lộ", 7, 9),
            (180.0, "Thu Phân", 22, 9),
            (195.0, "Hàn Lộ", 8, 10),
            (210.0, "Sương Giáng", 23, 10),
            (225.0, "Lập Đông", 7, 11),
            (240.0, "Tiểu Tuyết", 22, 11),
            (255.0, "Đại Tuyết", 6, 12),
            (270.0, "Đông Chí", 21, 12),
        ];
        for (deg, ten, ngay, thang) in chuan {
            let jd = find_tiet_khi_jd(deg, 2024)
                .unwrap_or_else(|| panic!("không tìm thấy {ten} ({deg}°) năm 2024"));
            let (_, m, d) = jd_to_ymd(jd);
            assert_eq!(
                (d, m),
                (ngay, thang),
                "{ten} ({deg}°) 2024 phải rơi {ngay}/{thang}, nhận {d}/{m}"
            );
        }
    }

    /// `TIET_KHI_24` phải nhất quán: 24 mục, kinh độ tăng đều 15°, và các mục
    /// chỉ số chẵn phải trùng khớp `TIET_KHI_TABLE` (12 tiết 節).
    #[test]
    fn bang_24_tiet_khi_nhat_quan_voi_bang_12_tiet() {
        assert_eq!(TIET_KHI_24.len(), 24);
        for (i, &(deg, _)) in TIET_KHI_24.iter().enumerate() {
            let mong_doi = (315 + 15 * i as u32) % 360;
            assert_eq!(deg, mong_doi, "mục {i} sai kinh độ");
        }
        for (i, &(deg, ten)) in TIET_KHI_TABLE.iter().enumerate() {
            assert_eq!(TIET_KHI_24[i * 2], (deg, ten), "tiết 節 thứ {i} không khớp");
        }
    }

    /// `tiet_month_index` phải nhất quán với `find_tiet_khi_jd`: ngày giao tiết
    /// đã thuộc tháng MỚI, ngày liền trước vẫn thuộc tháng CŨ.
    #[test]
    fn tiet_month_index_nhat_quan_voi_moc_tiet() {
        for nam in [2000i32, 2024, 2025, 2050] {
            for (i, &(deg, ten)) in TIET_KHI_TABLE.iter().enumerate() {
                let nam_tim = if deg == 285 { nam + 1 } else { nam };
                let jd = find_tiet_khi_jd(deg as f64, nam_tim).expect("tìm được tiết");
                let thang_moi = (i + 1) as u8;
                let thang_cu = if thang_moi == 1 { 12 } else { thang_moi - 1 };
                assert_eq!(
                    tiet_month_index(jd),
                    thang_moi,
                    "{ten} {nam}: ngày giao tiết phải thuộc tháng {thang_moi}"
                );
                assert_eq!(
                    tiet_month_index(jd - 1),
                    thang_cu,
                    "{ten} {nam}: ngày trước mốc phải còn thuộc tháng {thang_cu}"
                );
            }
        }
    }

    #[test]
    fn tiet_khi_12th_jd_is_tieu_han_in_next_calendar_year() {
        let jds = tiet_khi_jds_of_bt_year(2024);
        let (y, m, _d) = jd_to_ymd(jds[11]);
        assert_eq!(y, 2025);
        assert_eq!(m, 1);
    }
}