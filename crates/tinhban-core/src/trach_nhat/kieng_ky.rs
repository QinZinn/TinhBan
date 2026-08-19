//! Các ngày kiêng kỵ phổ biến: Tam Nương, Nguyệt Kỵ, Sát Chủ.
//!
//! Ba mục này **độc lập** với nhau và độc lập với Hoàng Đạo/Hắc Đạo — một ngày
//! có thể vừa Hoàng Đạo vừa Tam Nương. Vì vậy hàm ở đây trả về **danh sách**,
//! không quy về một điểm số.

use super::types::KiengKy;
use crate::{EarthlyBranch, LunarDate};

/// Ngày Âm lịch bị coi là Tam Nương: mùng 3, 7, 13, 18, 22, 27.
pub const NGAY_TAM_NUONG: [u8; 6] = [3, 7, 13, 18, 22, 27];

/// Ngày Âm lịch bị coi là Nguyệt Kỵ: mùng 5, 14, 23 (mọi tháng).
///
/// Dân gian gọi là "mùng năm, mười bốn, hai ba" — cộng các chữ số đều ra 5,
/// ứng với "trung cung" trong Cửu Cung.
pub const NGAY_NGUYET_KY: [u8; 3] = [5, 14, 23];

/// Bảng Sát Chủ: với mỗi **tháng Âm lịch** 1..12, liệt kê index Địa Chi
/// (0 = Tý … 11 = Hợi) của những ngày bị coi là Sát Chủ.
///
/// # Nguồn của bảng
///
/// Bảng này được **suy ra từ dữ liệu thật** chứ không chép từ một sách nào, vì
/// các bản "bảng Sát Chủ" lưu hành trên mạng mâu thuẫn nhau. Cách làm: với mỗi
/// tháng Âm lịch, quét **12 ngày liên tiếp** (đủ trọn 12 Địa Chi, mỗi Chi đúng
/// một lần) trên licham365.vn và ghi lại ngày nào bị gắn nhãn "Ngày sát chủ" —
/// lặp lại cho **2 năm Âm lịch độc lập (2024 và 2025)**, tổng 288 trang ngày.
/// Kết quả hai năm **trùng khớp 12/12 tháng**, xác nhận đây là bảng tra cố định
/// theo tháng Âm lịch (không phụ thuộc năm, không phụ thuộc Can của ngày).
///
/// # Vì sao mỗi tháng có tới 3 Chi
///
/// Các sách thường tách "Sát Chủ Dương" và "Sát Chủ Âm", mỗi bảng 1 Chi/tháng.
/// Bảng dưới đây là **hợp** của nhiều dòng truyền (khớp đúng những gì
/// licham365.vn đánh dấu), nên rộng hơn một bảng đơn lẻ. Ai chỉ muốn kiêng theo
/// một dòng cụ thể sẽ thấy bảng này báo nhiều ngày hơn mong đợi — đây là lựa
/// chọn có chủ ý để khớp nguồn đối chiếu, không phải lỗi.
///
/// Tháng 1 và tháng 12 chỉ có 2 Chi (không phải do quét thiếu — cả 12 Chi đều đã
/// được kiểm và chỉ 2 Chi được đánh dấu).
pub const BANG_SAT_CHU: [&[u8]; 12] = [
    &[0, 5],     // tháng 1  — Tý, Tỵ
    &[0, 1, 3],  // tháng 2  — Tý, Sửu, Mão
    &[1, 6, 7],  // tháng 3  — Sửu, Ngọ, Mùi
    &[3, 9, 10], // tháng 4  — Mão, Dậu, Tuất
    &[0, 4, 8],  // tháng 5  — Tý, Thìn, Thân
    &[4, 9, 10], // tháng 6  — Thìn, Dậu, Tuất
    &[1, 6, 11], // tháng 7  — Sửu, Ngọ, Hợi
    &[1, 3, 4],  // tháng 8  — Sửu, Mão, Thìn
    &[0, 1, 6],  // tháng 9  — Tý, Sửu, Ngọ
    &[3, 4, 9],  // tháng 10 — Mão, Thìn, Dậu
    &[2, 6, 7],  // tháng 11 — Dần, Ngọ, Mùi
    &[4, 9],     // tháng 12 — Thìn, Dậu
];

/// Ngày Âm lịch `lunar` có phải Tam Nương không.
pub fn la_tam_nuong(lunar: LunarDate) -> bool {
    NGAY_TAM_NUONG.contains(&lunar.day)
}

/// Ngày Âm lịch `lunar` có phải Nguyệt Kỵ không.
pub fn la_nguyet_ky(lunar: LunarDate) -> bool {
    NGAY_NGUYET_KY.contains(&lunar.day)
}

/// Ngày có Chi `chi_ngay`, nằm trong tháng Âm lịch của `lunar`, có phải Sát Chủ
/// không.
///
/// Tháng nhuận dùng chung bảng với tháng chính cùng số (ví dụ tháng 4 nhuận tra
/// cùng dòng với tháng 4).
pub fn la_sat_chu(lunar: LunarDate, chi_ngay: EarthlyBranch) -> bool {
    let Some(row) = BANG_SAT_CHU.get(lunar.month as usize - 1) else {
        return false; // tháng ngoài 1..=12 — không thể xảy ra với LunarDate hợp lệ
    };
    row.contains(&chi_ngay.index())
}

/// Tất cả điều kiêng kỵ áp dụng cho ngày này, theo thứ tự ổn định
/// Tam Nương → Nguyệt Kỵ → Sát Chủ. Trả vec rỗng nếu ngày "sạch".
pub fn kieng_ky_cua_ngay(lunar: LunarDate, chi_ngay: EarthlyBranch) -> Vec<KiengKy> {
    let mut out = Vec::new();
    if la_tam_nuong(lunar) {
        out.push(KiengKy::TamNuong);
    }
    if la_nguyet_ky(lunar) {
        out.push(KiengKy::NguyetKy);
    }
    if la_sat_chu(lunar, chi_ngay) {
        out.push(KiengKy::SatChu);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lunar(day: u8, month: u8) -> LunarDate {
        LunarDate {
            day,
            month,
            year: 2025,
            is_leap_month: false,
        }
    }

    #[test]
    fn tam_nuong_dung_6_ngay_moi_thang() {
        let hit: Vec<u8> = (1..=30).filter(|d| la_tam_nuong(lunar(*d, 5))).collect();
        assert_eq!(hit, vec![3, 7, 13, 18, 22, 27]);
    }

    #[test]
    fn nguyet_ky_dung_3_ngay_moi_thang() {
        let hit: Vec<u8> = (1..=30).filter(|d| la_nguyet_ky(lunar(*d, 5))).collect();
        assert_eq!(hit, vec![5, 14, 23]);
    }

    /// Bảng Sát Chủ phải hợp lệ: mọi index Chi trong 0..12, không trùng trong
    /// cùng một tháng, và đủ 12 dòng.
    #[test]
    fn bang_sat_chu_hop_le() {
        assert_eq!(BANG_SAT_CHU.len(), 12);
        for (i, row) in BANG_SAT_CHU.iter().enumerate() {
            assert!(!row.is_empty(), "tháng {} rỗng", i + 1);
            assert!(row.iter().all(|c| *c < 12), "tháng {} có Chi >= 12", i + 1);
            let mut s = row.to_vec();
            s.sort_unstable();
            s.dedup();
            assert_eq!(s.len(), row.len(), "tháng {} có Chi trùng", i + 1);
            assert!(row.windows(2).all(|w| w[0] < w[1]), "tháng {} chưa sắp xếp", i + 1);
        }
    }

    /// Một ngày có thể vừa Tam Nương vừa Sát Chủ — danh sách phải giữ cả hai,
    /// không "nuốt" mục nào.
    #[test]
    fn nhieu_kieng_ky_cung_luc_deu_duoc_liet_ke() {
        // Tháng 7 Âm, ngày Sửu, mùng 7 → Tam Nương + Sát Chủ.
        let kk = kieng_ky_cua_ngay(lunar(7, 7), EarthlyBranch::Suu);
        assert_eq!(kk, vec![KiengKy::TamNuong, KiengKy::SatChu]);

        // Mùng 5 tháng 7, ngày Hợi → Nguyệt Kỵ + Sát Chủ.
        let kk = kieng_ky_cua_ngay(lunar(5, 7), EarthlyBranch::Hoi);
        assert_eq!(kk, vec![KiengKy::NguyetKy, KiengKy::SatChu]);

        // Ngày "sạch": mùng 8 tháng 7, ngày Dần.
        assert!(kieng_ky_cua_ngay(lunar(8, 7), EarthlyBranch::Dan).is_empty());
    }
}
