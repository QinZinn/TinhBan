//! Hoàng Đạo / Hắc Đạo — an 12 vị Thần vào 12 giờ (trực giờ) và vào ngày
//! (trực nhật).
//!
//! # Quy tắc chung
//!
//! Vòng 12 vị Thần ([`ThanSat`]) luôn chạy theo thứ tự cố định. Việc duy nhất
//! phải tính là **điểm khởi** của vòng — vị trí Địa Chi mà Thanh Long đóng:
//!
//! ```text
//! khởi_Thanh_Long = (2 × chi_gốc + 8) mod 12
//! ```
//!
//! rồi vị Thần tại một Địa Chi `x` là `ThanSat[(x − khởi) mod 12]`.
//!
//! Bảng truyền thống mà công thức trên tái tạo đúng:
//!
//! | Chi gốc      | Thanh Long đóng tại |
//! |--------------|---------------------|
//! | Tý, Ngọ      | Thân                |
//! | Sửu, Mùi     | Tuất                |
//! | Dần, Thân    | Tý                  |
//! | Mão, Dậu     | Dần                 |
//! | Thìn, Tuất   | Thìn                |
//! | Tỵ, Hợi      | Ngọ                 |
//!
//! # ⚠ "Chi gốc" là gì thì tuỳ cấp — đây là chỗ dễ sai nhất
//!
//! - **Trực giờ** (giờ Hoàng Đạo trong ngày): chi gốc = **Chi của NGÀY**.
//! - **Trực nhật** (ngày Hoàng Đạo/Hắc Đạo): chi gốc = **Chi của THÁNG ÂM LỊCH**.
//!
//! Và lưu ý tiếp: tháng dùng ở đây là **tháng Âm lịch**, KHÁC với tháng tiết khí
//! mà [`super::truc`] dùng cho 12 Trực. Hai cấp dùng hai loại tháng khác nhau —
//! không phải nhầm lẫn, mà là đúng theo lịch vạn niên Việt Nam (xem phần đối
//! chiếu trong `trach_nhat/README.md`).

use super::types::{HoangDaoHacDao, HourRange, ThanSat};
use crate::{CanChi, EarthlyBranch};

/// Vị trí (index Địa Chi 0..11) mà Thanh Long đóng, cho một `chi gốc` cho trước.
///
/// Xem bảng trong doc của module. Công thức `(2b + 8) mod 12` đã được đối chiếu
/// khớp 100% với cả 6 dòng của bảng truyền thống.
fn khoi_thanh_long(chi_goc: EarthlyBranch) -> u8 {
    (2 * chi_goc.index() + 8) % 12
}

/// Vị Thần trực tại Địa Chi `vi_tri`, khi vòng khởi từ `chi_goc`.
fn than_tai(chi_goc: EarthlyBranch, vi_tri: EarthlyBranch) -> ThanSat {
    let khoi = khoi_thanh_long(chi_goc);
    let idx = (vi_tri.index() + 12 - khoi) % 12;
    ThanSat::from_index(idx).expect("idx luôn < 12 do mod 12")
}

/// Giờ đồng hồ bắt đầu khung giờ của một Địa Chi: Tý = 23, Sửu = 1, Dần = 3, …
///
/// Giờ Tý vắt qua nửa đêm (23:00–00:59) nên là khung duy nhất có
/// `start_hour > end_hour`.
fn gio_bat_dau(branch: EarthlyBranch) -> u8 {
    match branch.index() {
        0 => 23,
        i => i * 2 - 1,
    }
}

/// An 12 vị Thần vào 12 khung giờ của một ngày có Can Chi `day`.
///
/// Trả về mảng theo thứ tự Địa Chi Tý → Hợi (KHÔNG phải theo thứ tự thời gian
/// trong ngày — giờ Tý đứng đầu dù bắt đầu lúc 23:00).
pub fn cac_gio_trong_ngay(day: CanChi) -> Vec<HourRange> {
    (0..12u8)
        .map(|i| {
            let branch = EarthlyBranch::from_index(i).expect("i < 12");
            let start = gio_bat_dau(branch);
            HourRange {
                branch,
                than: than_tai(day.branch, branch),
                start_hour: start,
                end_hour: (start + 1) % 24,
            }
        })
        .collect()
}

/// Ngày Hoàng Đạo hay Hắc Đạo: an vòng 12 Thần theo **Chi của tháng Âm lịch**,
/// rồi đọc vị Thần đóng tại **Chi của ngày**.
pub fn danh_gia_ngay(day: CanChi, thang_am_branch: EarthlyBranch) -> HoangDaoHacDao {
    let than = than_tai(thang_am_branch, day.branch);
    HoangDaoHacDao {
        than,
        is_hoang_dao: than.is_hoang_dao(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use EarthlyBranch::*;

    /// Bảng truyền thống "Thanh Long đóng tại đâu" — 6 dòng, mỗi dòng 2 Chi.
    #[test]
    fn bang_khoi_thanh_long_khop_truyen_thong() {
        let bang = [
            (Ty, Than),
            (Ngo, Than),
            (Suu, Tuat),
            (Mui, Tuat),
            (Dan, Ty),
            (Than, Ty),
            (Mao, Dan),
            (Dau, Dan),
            (Thin, Thin),
            (Tuat, Thin),
            (Ty2, Ngo),
            (Hoi, Ngo),
        ];
        for (goc, mong_doi) in bang {
            assert_eq!(
                khoi_thanh_long(goc),
                mong_doi.index(),
                "chi gốc {:?}: Thanh Long phải đóng tại {:?}",
                goc,
                mong_doi
            );
        }
    }

    /// Mỗi ngày phải có đúng 6 giờ Hoàng Đạo và 6 giờ Hắc Đạo, và 12 vị Thần
    /// xuất hiện đúng một lần.
    #[test]
    fn moi_ngay_co_dung_6_gio_tot_6_gio_xau() {
        for i in 0..12u8 {
            let day = CanChi {
                stem: crate::HeavenlyStem::Giap,
                branch: EarthlyBranch::from_index(i).unwrap(),
            };
            let gio = cac_gio_trong_ngay(day);
            assert_eq!(gio.len(), 12);
            let tot = gio.iter().filter(|g| g.is_hoang_dao()).count();
            assert_eq!(tot, 6, "chi ngày index {i}: phải có 6 giờ Hoàng Đạo");

            let mut thans: Vec<u8> = gio.iter().map(|g| g.than.index()).collect();
            thans.sort_unstable();
            assert_eq!(
                thans,
                (0..12).collect::<Vec<u8>>(),
                "12 vị Thần phải xuất hiện đúng 1 lần"
            );
        }
    }

    /// Khung giờ phải đúng: Tý 23:00–00:59, Sửu 01:00–02:59, …, Hợi 21:00–22:59.
    #[test]
    fn khung_gio_dia_chi_dung() {
        let day = CanChi {
            stem: crate::HeavenlyStem::Giap,
            branch: EarthlyBranch::Ty,
        };
        let gio = cac_gio_trong_ngay(day);
        let mong_doi = [
            (Ty, 23u8, 0u8),
            (Suu, 1, 2),
            (Dan, 3, 4),
            (Mao, 5, 6),
            (Thin, 7, 8),
            (Ty2, 9, 10),
            (Ngo, 11, 12),
            (Mui, 13, 14),
            (Than, 15, 16),
            (Dau, 17, 18),
            (Tuat, 19, 20),
            (Hoi, 21, 22),
        ];
        for (g, (b, s, e)) in gio.iter().zip(mong_doi) {
            assert_eq!(g.branch, b);
            assert_eq!(g.start_hour, s, "{:?} start", b);
            assert_eq!(g.end_hour, e, "{:?} end", b);
        }
    }

    /// Ngày Dần/Thân → giờ Hoàng Đạo là Tý, Sửu, Thìn, Tỵ, Mùi, Tuất
    /// (đối chiếu trực tiếp với lịch vạn niên).
    #[test]
    fn ngay_dan_than_co_gio_hoang_dao_dung() {
        for chi in [Dan, Than] {
            let day = CanChi {
                stem: crate::HeavenlyStem::Mau,
                branch: chi,
            };
            let tot: Vec<EarthlyBranch> = cac_gio_trong_ngay(day)
                .into_iter()
                .filter(|g| g.is_hoang_dao())
                .map(|g| g.branch)
                .collect();
            assert_eq!(tot, vec![Ty, Suu, Thin, Ty2, Mui, Tuat], "ngày {:?}", chi);
        }
    }
}
