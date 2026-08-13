//! An 14 chính tinh theo quy tắc Bắc Tông (theo `lasotuvi.App::lapDiaBan`).
//!
//! # Tóm tắt quy tắc
//!
//! 1. **An Tử Vi** (theo Cục + ngày sinh âm lịch): thuật toán `timTuVi` của
//!    `lasotuvi.AmDuong`:
//!    - Bắt đầu từ cung **Dần** (index 2), gán `cuc_so` = `chart.cuc.so()`.
//!    - Vòng lặp `while cuc < ngay_sinh_am_lich: cuc += cuc_so; cung_dan += 1`.
//!    - `delta = cuc - ngay_sinh` — lệch **chẵn ⇒ tiến** (dịch thuận), lẻ ⇒
//!      lùi: nếu `delta % 2 == 1` thì `delta = -delta`.
//!    - Vị trí Tử Vi = `dichCung(cung_dan, delta)`.
//! 2. **Tử Vi tinh hệ** (5 sao còn lại của chuỗi Bắc Đẩu): khoảng cách cố
//!    định từ vị trí Tử Vi (theo lasotuvi):
//!    - Liêm Trinh  = Tử Vi + 4
//!    - Thiên Đồng  = Tử Vi + 7
//!    - Vũ Khúc     = Tử Vi + 8
//!    - Thái Dương  = Tử Vi + 9
//!    - Thiên Cơ     = Tử Vi + 11
//! 3. **Thiên Phủ tinh hệ** (8 sao Thiên Phủ-Phá Quân):
//!    - Thiên Phủ = `dichCung(Dần=3_1based, 3 - viTriTuVi_1based)` trong Ho.
//!      Port 0-based: `ThienPhu = dichCung(Dần, 3 - (TuVi + 1) - 1)` — chờ đã.
//!      Quy ước Ho 1-based: `viTriThienPhu = dichCung(3, 3 - viTriTuVi)`,
//!      → 3 + 3 - viTriTuVi = 6 - viTriTuVi. Port 0-based:
//!      `(2 + 3 - (tuvi_0) - 1) = (4 - tuvi_0)` → hmm. Đơn giản hơn:
//!      Thiên Phủ luôn ở vị trí "đối xứng" với Tử Vi qua trục Sửu-Mùi: nếu Tử Vi
//!      ở Dần(index 2), Thiên Phủ ở Thân(index 8) — index cộng = 10≡ -2.
//!      Tổng quát: `ThienPhu = (4 - tuvi_0) mod 12` cho 0-based.
//!      Cách ít rủi ro: dùng công thức chính:
//!      ThienPhu_0 = dichCung(Dần_0, 3 - 1 - viTriTuVi_1 - 1) = dichCung(Dần, 4 - viTriTuVi)
//!      với viTriTuVi lần lượt 1-based rồi port:
//!      ho: thien_phu_1 = 3 + 3 - viTriTuVi_1 = 6 - viTriTuVi_1
//!      → port 0-based = thien_phu_1 - 1 = 5 - viTriTuVi_1 = 5 - (tuvi_0 + 1) = 4 - tuvi_0. →
//!      → `ThienPhu_0 = (4 - tuvi_0) mod 12` ✓.
//!    - Các sao Thiên Phủ tinh hệ sau, theo khoảng cách cố định từ Thiên Phủ
//!      (theo lasotuvi): Thái Âm +1, Tham Lang +2, Cự Môn +3, Thiên Tướng +4,
//!      Thiên Lương +5, Thất Sát +6, Phá Quân +10 (nhảy qua theo quy ước).

use crate::EarthlyBranch;
use crate::tuvi::dich_cung;
use crate::tuvi::stars::Sao;
use crate::tuvi::types::{TuViChart, TuViError};

/// An 14 chính tinh vào `chart.palaces[i].stars` (i là 12 cung theo Địa Chi).
pub fn an_chinh_tinh(chart: &mut TuViChart) -> Result<(), TuViError> {
    let ngay_sinh_am = chart.lunar.day as i64;
    let cuc_so = chart.cuc.so as i64;

    // 1. Vị trí Tử Vi — port `timTuVi(cuc_so, ngay_sinh_am)` của lasotuvi.
    let tuvi_idx_0 = find_tu_vi(cuc_so, ngay_sinh_am) as i64;
    let tuvi_branch = EarthlyBranch::from_index(tuvi_idx_0 as u8).unwrap();

    // Cung Mệnh_redefined for placement:
    if let Some(p) = chart.palace_mut(tuvi_branch) {
        p.add_star(Sao::TuVi);
    }

    // 2. Tử Vi tinh hệ — khoảng cách cố định từ Tử Vi.
    let tinh_he_offsets: &[(Sao, i64)] = &[
        (Sao::LiemTrinh, 4),
        (Sao::ThienDong, 7),
        (Sao::VuKhuc, 8),
        (Sao::ThaiDuong, 9),
        (Sao::ThienCo, 11),
    ];
    for (sao, off) in tinh_he_offsets {
        let b = dich_cung(tuvi_branch, *off);
        if let Some(p) = chart.palace_mut(b) {
            p.add_star(*sao);
        }
    }

    // 3. Thiên Phủ = (4 - tuvi_idx_0) mod 12 (port 0-based — giải thích ở module doc).
    let thien_phu_idx = (4 - tuvi_idx_0).rem_euclid(12);
    let thien_phu = EarthlyBranch::from_index(thien_phu_idx as u8).unwrap();
    if let Some(p) = chart.palace_mut(thien_phu) {
        p.add_star(Sao::ThienPhu);
    }

    // 4. Thiên Phủ tinh hệ — khoảng cách cố định từ Thiên Phủ.
    let phu_he_offsets: &[(Sao, i64)] = &[
        (Sao::ThaiAm, 1),
        (Sao::ThamLang, 2),
        (Sao::CuMon, 3),
        (Sao::ThienTuong, 4),
        (Sao::ThienLuong, 5),
        (Sao::ThatSat, 6),
        (Sao::PhaQuan, 10),
    ];
    for (sao, off) in phu_he_offsets {
        let b = dich_cung(thien_phu, *off);
        if let Some(p) = chart.palace_mut(b) {
            p.add_star(*sao);
        }
    }

    Ok(())
}

/// Tìm vị trí sao Tử Vi theo Cục + ngày sinh âm lịch.
///
/// Port `lasotuvi.AmDuong.timTuVi` (1-based) sang 0-based:
///  - bắt đầu từ cung Dần (0-based index 2).
///  - tăng lên 1 cung + tăng cuc lên `cuc_so` đỉt cho tới khi `cuc >= ngày`.
///  - lệch = cuc - ngày; nếu lệch lẻ thì "lùi" (đổi dấu); nếu chẵn thì "tiến".
///  - vị trí Tử Vi = dichCung(Dần, lệch_sau_khi_xử_lý_dấu).
fn find_tu_vi(cuc_so: i64, ngay_sinh: i64) -> u8 {
    let mut cung_dan = 2i64; // Dần (0-based)
    let cuc_ban_dau = cuc_so;
    let mut cuc = cuc_so;
    while cuc < ngay_sinh {
        cuc += cuc_ban_dau;
        cung_dan += 1;
    }
    let mut sai_lech = cuc - ngay_sinh;
    if sai_lech % 2 == 1 {
        sai_lech = -sai_lech;
    }
    ((cung_dan + sai_lech).rem_euclid(12)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuvi::cuc::{tinh_cuc, Cuc};
    use crate::tuvi::stars::SaoCategory;
    use crate::tuvi::{Gender, lap_la_so};
    use crate::BirthMoment;
    use chrono::NaiveDate;

    #[test]
    fn chinh_tinh_count_is_14_for_any_chart() {
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
            hour: 12,
            minute: 0,
        };
        let chart = lap_la_so(birth, Gender::Nam).unwrap();
        let count = chart
            .palaces
            .iter()
            .map(|p| p.stars.iter().filter(|s| s.category() == SaoCategory::ChinhTinh).count())
            .sum::<usize>();
        assert_eq!(count, 14);
    }

    #[test]
    fn thien_phu_is_opposite_of_tu_vi_via_suu_mui_axis() {
        // Tử Vi và Thiên Phủ luôn đối xứng qua trục Sửu-Mùi: tổng 2 index ≡ 4
        // (mod 12). Ví dụ Tử Vi ở Dần (2) thì Thiên Phủ ở Thân (8); 2+8=10≡-2.
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(1991, 10, 24).unwrap(),
            hour: 7,
            minute: 30,
        };
        let chart = lap_la_so(birth, Gender::Nam).unwrap();
        for p in chart.palaces.iter() {
            if p.has_star(Sao::TuVi) {
                let tuvi = p.branch.index() as i64;
                // Tìm Thiên Phủ ở cung tương ứng
                for q in chart.palaces.iter() {
                    if q.has_star(Sao::ThienPhu) {
                        let thienphu = q.branch.index() as i64;
                        assert_eq!((tuvi + thienphu).rem_euclid(12), 4,
                            "Tử Vi ({tuvi}) + Thiên Phủ ({thienphu}) ≡ 4 mod 12");
                    }
                }
            }
        }
    }

    #[test]
    fn find_tu_vi_returns_valid_index() {
        for cuc in [2, 3, 4, 5, 6] {
            for day in 1..=30 {
                let idx = find_tu_vi(cuc as i64, day as i64);
                assert!(idx < 12, "cuc={cuc} day={day} → idx {idx}");
            }
        }
        let _ = tinh_cuc;
        let _ = Cuc { so: 2, hanh: crate::NguHanh::Thuy };
    }
}