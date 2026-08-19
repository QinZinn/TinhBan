//! 12 Trực (Kiến Trừ Thập Nhị Khách / 建除十二神).
//!
//! # Quy tắc
//!
//! Vòng 12 Trực chạy theo thứ tự cố định Kiến → Trừ → Mãn → Bình → Định → Chấp
//! → Phá → Nguy → Thành → Thu → Khai → Bế, mỗi ngày tiến một bước. Neo của vòng:
//!
//! > Trực **Kiến** rơi vào ngày có **Chi trùng với Chi của tháng**.
//!
//! nên:
//!
//! ```text
//! truc = (chi_ngày − chi_tháng) mod 12
//! ```
//!
//! # ⚠ "Chi tháng" ở đây là tháng TIẾT KHÍ, không phải tháng Âm lịch
//!
//! Đây là chỗ sai phổ biến nhất khi tự implement 12 Trực. Tháng dùng cho Trực là
//! **tháng tiết khí** (節月): tháng Dần bắt đầu từ Lập Xuân, tháng Mão từ Kinh
//! Trập, … — xem [`crate::bat_tu::tiet_khi::tiet_month_branch_index`]. Mốc tiết
//! khí không trùng mùng 1 Âm lịch, nên hai loại tháng lệch nhau vài ngày mỗi
//! tháng.
//!
//! Đối chiếu 174 ngày với licham365.vn: dùng tháng **tiết khí** khớp 172/174,
//! dùng tháng **Âm lịch** chỉ khớp 149/174. (2 ngày còn lại là mâu thuẫn nội bộ
//! của chính licham365 — xem `trach_nhat/README.md`.)
//!
//! Lưu ý: ngày Hoàng Đạo/Hắc Đạo thì ngược lại — dùng tháng **Âm lịch**. Xem
//! [`super::hoang_dao`].
//!
//! # "Trùng Trực" tự nhiên xuất hiện
//!
//! Truyền thống nói vào ngày giao tiết, Trực **lặp lại** một ngày. Công thức
//! trên tự sinh ra hiện tượng đó mà không cần xử lý riêng: qua mốc tiết,
//! `chi_tháng` tăng 1 trong khi `chi_ngày` cũng tăng 1, nên hiệu số giữ nguyên.

use super::types::Truc;
use crate::bat_tu::tiet_khi::tiet_month_branch_index;
use crate::CanChi;

/// Trực của ngày có Can Chi `day`, nằm trong ngày Julian `jd`.
pub fn truc_of_day(day: CanChi, jd: i64) -> Truc {
    let chi_thang = tiet_month_branch_index(jd);
    let idx = (day.branch.index() + 12 - chi_thang) % 12;
    Truc::from_index(idx).expect("idx luôn < 12 do mod 12")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EarthlyBranch, HeavenlyStem};

    /// Ngày có Chi trùng Chi tháng tiết khí → Trực Kiến; lệch 1 → Trừ; …
    #[test]
    fn hieu_chi_ngay_tru_chi_thang_ra_dung_truc() {
        for chi_thang in 0..12u8 {
            for lech in 0..12u8 {
                let chi_ngay = (chi_thang + lech) % 12;
                let idx = (chi_ngay + 12 - chi_thang) % 12;
                assert_eq!(
                    Truc::from_index(idx).unwrap().index(),
                    lech,
                    "chi tháng {chi_thang}, lệch {lech}"
                );
            }
        }
    }

    /// 12 Trực phải phủ đúng 12 giá trị, không trùng, không thiếu.
    #[test]
    fn du_12_truc_khong_trung() {
        let mut seen = [false; 12];
        for i in 0..12u8 {
            let t = Truc::from_index(i).unwrap();
            assert!(!seen[t.index() as usize], "Trực {:?} lặp", t);
            seen[t.index() as usize] = true;
        }
        assert!(seen.iter().all(|x| *x));
        assert!(Truc::from_index(12).is_none());
    }

    /// Ngày liên tiếp (cùng tháng tiết khí) phải cho Trực liên tiếp.
    #[test]
    fn truc_tien_mot_buoc_moi_ngay() {
        // JD giữa tháng Mão 2024 (Kinh Trập 5/3 → Thanh Minh 4/4): 10–20/3/2024.
        let jd_10_3_2024 = crate::astronomy::jd_from_date(10, 3, 2024);
        let mut truoc: Option<Truc> = None;
        for off in 0..10 {
            let jd = jd_10_3_2024 + off;
            let day = crate::canchi::day_can_chi(jd);
            let t = truc_of_day(day, jd);
            if let Some(p) = truoc {
                assert_eq!(
                    t.index(),
                    (p.index() + 1) % 12,
                    "jd {jd}: Trực phải tiến 1 bước từ {:?}",
                    p
                );
            }
            truoc = Some(t);
        }
        let _ = HeavenlyStem::Giap;
        let _ = EarthlyBranch::Ty;
    }
}
