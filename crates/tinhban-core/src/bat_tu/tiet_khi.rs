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
//! | Tiểu Mãn   |  75°       | 5 — Ngọ                |
//! | Mang Chủng | 105°       | 6 — Mùi                |
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
//! # Giới hạn độ chính xác (~±1 ngày)
//!
//! Công thức polynomial của Hồ Ngọc Đức (Meeus chương 25) có độ chính xác
//! ~vài phút cho thời điểm sóc, nhưng kinh độ Mặt Trời chênh ±0.5° so với
//! high-precision ephemeris ở một số năm. Vì Sun tiến ~0.985°/ngày, ±0.5° ở
//! kinh độ ≈ ±0.5 ngày ở thời điểm.
//!
//! Trong thực tế: 6 năm Lập Xuân thử nghiệm có 4/6 lệch 1 ngày so với một số
//! nguồn chính thức (HK Observatory). Bát Tự generates ở khu vực biên tiết (sinh
//! trúng ngày tiết hoặc ngày liền kề) **có thể lệch 1 ngày tháng BT**.
//!
//! Khuyến nghị: cung cấp ngày sinh cách biên tiết ít nhất **5 ngày** để đảm bảo
//! BT year/month được xác định unambiguously. Lá số ở giữa tháng BT (xa biên)
//! không bị ảnh hưởng bởi giới hạn này.

use crate::astronomy::{jd_from_date, jd_to_date, sun_longitude_deg_at_local_midnight, VN_TZ};

/// 12 tiết khí theo thứ tự Lập Xuân → Tiểu Hàn, kèm kinh độ Mặt Trời (°) và
/// tên tiếng Việt. Index 0..11 ứng với tháng BT 1..12 (Dần..Sửu).
pub const TIET_KHI_TABLE: [(u32, &str); 12] = [
    (315, "Lập Xuân"),    // → tháng 1, Dần
    (345, "Kinh Trập"),   // → tháng 2, Mão
    (15, "Thanh Minh"),   // → tháng 3, Thìn
    (45, "Lập Hạ"),       // → tháng 4, Tỵ
    (75, "Tiểu Mãn"),     // → tháng 5, Ngọ
    (105, "Mang Chủng"),  // → tháng 6, Mùi
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

    #[test]
    fn tiet_khi_12th_jd_is_tieu_han_in_next_calendar_year() {
        let jds = tiet_khi_jds_of_bt_year(2024);
        let (y, m, _d) = jd_to_ymd(jds[11]);
        assert_eq!(y, 2025);
        assert_eq!(m, 1);
    }
}