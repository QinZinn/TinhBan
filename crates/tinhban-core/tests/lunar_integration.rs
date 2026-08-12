//! Integration tests cho lõi âm lịch & Can Chi (`tinhban-core`).
//!
//! Dữ liệu đối chiếu nằm trong `test_data.rs` (auto-sinh từ prototype Python
//! dùng cùng thuật toán — xem `tinhban-core/README.md` mục "Quy trình đối
//! chiếu"). Toàn bộ test ở đây chưa quyết tính tuyệt đối là duyệt lich.vn — mà
//! là kết hợp:
//!  - 31 vector Tết âm lịch (so khớp với nhiều nguồn chính thức VN),
//!  - 5 vector từ `kunkka19xx/lunar` published (độc lập),
//!  - 11 vector tháng nhuận đã biết (kiểm tra 2 chiều S2L và L2S),
//!  - 80 vector roundtrip S2L→L2S (kiểm tra tính nhất quán trong của thuật toán
//!    Hồ Ngọc Đức — nếu hai chiều lệch nhau, thuật toán hoặc port có bug).

use chrono::NaiveDate;
use tinhban_core::{
    can_chi_display, day_can_chi, hour_can_chi, lunar_to_solar, month_can_chi,
    nguhanh_of_branch, nguhanh_of_stem, solar_to_lunar, year_can_chi, BirthMoment,
    EarthlyBranch, HeavenlyStem, LunarDate, LunarError, NguHanh,
};

#[path = "test_data.rs"]
mod test_data;

// =========================================================================
// 1. Test Tết (Lunar 1/1) — 31 vectors
// =========================================================================

#[test]
fn tet_solar_to_lunar_returns_day1_month1() {
    for &(yyyy, td, tm) in test_data::TET_CASES {
        let solar = NaiveDate::from_ymd_opt(yyyy as i32, tm as u32, td as u32).unwrap();
        let lunar = solar_to_lunar(solar).unwrap();
        assert_eq!(
            lunar,
            LunarDate {
                day: 1,
                month: 1,
                year: yyyy as i32,
                is_leap_month: false,
            },
            "Tết {} = {}-{}-{} (Dương) phải ứng với 1/1/{} âm — algorithm returned {:?}",
            yyyy, td, tm, yyyy, yyyy, lunar,
        );
    }
}

#[test]
fn tet_lunar_to_solar_returns_inverse() {
    for &(yyyy, td, tm) in test_data::TET_CASES {
        let lunar = LunarDate {
            day: 1,
            month: 1,
            year: yyyy as i32,
            is_leap_month: false,
        };
        let solar = lunar_to_solar(lunar).unwrap();
        assert_eq!(
            solar,
            NaiveDate::from_ymd_opt(yyyy as i32, tm as u32, td as u32).unwrap(),
            "L2S(1/1/{}) phải ra {}-{}-{} (Dương) — algorithm returned {}",
            yyyy, td, tm, yyyy, solar,
        );
    }
}

// =========================================================================
// 2. Kunkka19xx published external tests — independent external cross-check
// =========================================================================

#[test]
fn kunkka19xx_external_cases_match() {
    for &(d, m, y, ed, em, ey, eleap) in test_data::KUNKKA_CASES {
        let solar = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap();
        let lunar = solar_to_lunar(solar).unwrap();
        assert_eq!(
            lunar,
            LunarDate {
                day: ed,
                month: em,
                year: ey as i32,
                is_leap_month: eleap,
            },
            "kunkka19xx external test ({:02}/{:02}/{}) = expected {:02}/{:02}/{}{}",
            d, m, y, ed, em, ey, if eleap { " nhuận" } else { "" }
        );
    }
}

// =========================================================================
// 3. Rằm tháng Giêng (15/1 lunar) — full-moon-day anchor
// =========================================================================

#[test]
fn full_moon_jan_returns_day15_month1() {
    for &(d, m, y, ed, em, ey) in test_data::FULL_MOON_JAN {
        let solar = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap();
        let lunar = solar_to_lunar(solar).unwrap();
        assert_eq!(
            lunar,
            LunarDate {
                day: ed,
                month: em,
                year: ey as i32,
                is_leap_month: false,
            },
            "Rằm tháng Giêng {} = {}-{}-{} (Dương) phải ứng với 15/1/{}",
            ey, d, m, y, ey
        );
    }
}

// =========================================================================
// 4. Lunar leap months — full roundtrip S2L + L2S
// =========================================================================

#[test]
fn leap_month_l2s_s2l_roundtrip() {
    for &(ld, lm, ly, lleap, sd, sm, sy) in test_data::LEAP_MONTH_CASES {
        let lunar = LunarDate {
            day: ld,
            month: lm,
            year: ly as i32,
            is_leap_month: lleap,
        };
        let solar = lunar_to_solar(lunar).unwrap();
        let expected_solar = NaiveDate::from_ymd_opt(sy as i32, sm as u32, sd as u32).unwrap();
        assert_eq!(
            solar, expected_solar,
            "L2S({}/{}/{}) nhuận phải ra {}-{}-{} Dương — nhận {}",
            ld, lm, ly, sd, sm, sy, solar,
        );
        // Ngược lại: S2L(solar) phải trả lại lunar gốc
        let roundtrip = solar_to_lunar(solar).unwrap();
        assert_eq!(
            roundtrip, lunar,
            "S2L → L2S roundtrip mismatch cho {:02}/{:02}/{} → solar {} → lunar {:?}",
            sd, sm, sy, solar, roundtrip,
        );
    }
}

// =========================================================================
// 5. Year Can Chi —全面的 диапазонตรวจสอบ
// =========================================================================

#[test]
fn year_can_chi_returns_expected() {
    for &(y, stem_i, branch_i) in test_data::YEAR_CAN_CHI {
        let cc = year_can_chi(y).unwrap();
        assert_eq!(cc.stem.index(), stem_i, "year {} stem index", y);
        assert_eq!(cc.branch.index(), branch_i, "year {} branch index", y);
    }
}

#[test]
fn year_can_chi_well_known_anchors() {
    // Sanity check: những năm thường thấy trong văn hoá Việt Nam.
    let cases: &[(i32, &str)] = &[
        (2024, "Giáp Thìn"),
        (2025, "Ất Tỵ"),
        (2026, "Bính Ngọ"),
        (2000, "Canh Thìn"),
        (1996, "Bính Tý"),
        (1900, "Canh Tý"),
    ];
    for &(y, expected) in cases {
        let cc = year_can_chi(y).unwrap();
        let s = can_chi_display(cc);
        assert_eq!(s, expected, "year {}", y);
    }
}

// =========================================================================
// 6. Day Can Chi — chu kỳ 60 Giáp Tý + anchor với sample dates
// =========================================================================

#[test]
fn day_can_chi_60_cycle_consistency() {
    // Hai ngày cách nhau 60 ngày Dương lịch必有 cùng Can–Chi (chu kỳ 60 Giáp Tý).
    let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    for offset in 0..400 {
        let d1 = start + chrono::Duration::days(offset);
        let d2 = start + chrono::Duration::days(offset + 60);
        let c1 = day_can_chi(d1).unwrap();
        let c2 = day_can_chi(d2).unwrap();
        assert_eq!(c1, c2, "day {} vs {}+60 should match Can–Chi", d1, d2);
    }
}

#[test]
fn day_can_chi_consecutive_days_advance_by_one_step() {
    let start = NaiveDate::from_ymd_opt(2024, 5, 5).unwrap();
    for offset in 0..60 {
        let d1 = start + chrono::Duration::days(offset);
        let d2 = start + chrono::Duration::days(offset + 1);
        let c1 = day_can_chi(d1).unwrap();
        let c2 = day_can_chi(d2).unwrap();
        // Stem advances by 1 (mod 10), branch by 1 (mod 12).
        let expected_stem = HeavenlyStem::from_index((c1.stem.index() + 1) % 10).unwrap();
        let expected_branch =
            EarthlyBranch::from_index((c1.branch.index() + 1) % 12).unwrap();
        assert_eq!(c2.stem, expected_stem, "day {} → {} stem", d1, d2);
        assert_eq!(c2.branch, expected_branch, "day {} → {} branch", d1, d2);
    }
}

// =========================================================================
// 7. Month Can Chi — quy tắc Ngũ Thử Độn
// =========================================================================

#[test]
fn month_can_chi_branch_is_correct_12month_sequence() {
    // Dần, Mão, Thìn, Tỵ, Ngọ, Mùi, Thân, Dậu, Tuất, Hợi, Tý, Sửu.
    let branches = [
        EarthlyBranch::Dan,
        EarthlyBranch::Mao,
        EarthlyBranch::Thin,
        EarthlyBranch::Ty2,
        EarthlyBranch::Ngo,
        EarthlyBranch::Mui,
        EarthlyBranch::Than,
        EarthlyBranch::Dau,
        EarthlyBranch::Tuat,
        EarthlyBranch::Hoi,
        EarthlyBranch::Ty,
        EarthlyBranch::Suu,
    ];
    for (i, b) in branches.iter().enumerate() {
        let cc = month_can_chi(HeavenlyStem::Giap, (i + 1) as u8).unwrap();
        assert_eq!(cc.branch, *b, "month {}", i + 1);
    }
}

#[test]
fn month_can_chi_ngu_thu_don_with_year_2024() {
    // Năm Giáp Thìn (= 2024): month 1 = Bính Dần.
    let cc = month_can_chi(HeavenlyStem::Giap, 1).unwrap();
    assert_eq!(&can_chi_display(cc), "Bính Dần");
    // month 2: stem = Bính + 1 = Đinh, branch = Mão
    let cc = month_can_chi(HeavenlyStem::Giap, 2).unwrap();
    assert_eq!(&can_chi_display(cc), "Đinh Mão");
}

// =========================================================================
// 8. Hour Can Chi — 12 giờ Địa Chi + Ngũ Thử Độn Thời
// =========================================================================

#[test]
fn hour_can_chi_branch_sequence() {
    // Test 24 giờ: 23 → Tý, 0 → Tý, 1-2 → Sửu, ....
    let cases: &[(u8, EarthlyBranch)] = &[
        (0, EarthlyBranch::Ty),
        (1, EarthlyBranch::Suu),
        (2, EarthlyBranch::Suu),
        (3, EarthlyBranch::Dan),
        (4, EarthlyBranch::Dan),
        (5, EarthlyBranch::Mao),
        (6, EarthlyBranch::Mao),
        (7, EarthlyBranch::Thin),
        (8, EarthlyBranch::Thin),
        (9, EarthlyBranch::Ty2),
        (10, EarthlyBranch::Ty2),
        (11, EarthlyBranch::Ngo),
        (12, EarthlyBranch::Ngo),
        (13, EarthlyBranch::Mui),
        (14, EarthlyBranch::Mui),
        (15, EarthlyBranch::Than),
        (16, EarthlyBranch::Than),
        (17, EarthlyBranch::Dau),
        (18, EarthlyBranch::Dau),
        (19, EarthlyBranch::Tuat),
        (20, EarthlyBranch::Tuat),
        (21, EarthlyBranch::Hoi),
        (22, EarthlyBranch::Hoi),
        (23, EarthlyBranch::Ty),
    ];
    for &(h, b) in cases {
        let cc = hour_can_chi(HeavenlyStem::Giap, h).unwrap();
        assert_eq!(cc.branch, b, "hour {}", h);
    }
}

#[test]
fn hour_can_chi_ngu_thu_don_thoi() {
    // Ngày Giáp → giờ Tý Giáp Tý; ngày Ất → Bính Tý; ... (đối chiếu bảng Ngũ Thử Độn Thời)
    let table: &[(HeavenlyStem, HeavenlyStem)] = &[
        (HeavenlyStem::Giap, HeavenlyStem::Giap),
        (HeavenlyStem::Ky, HeavenlyStem::Giap),
        (HeavenlyStem::At, HeavenlyStem::Binh),
        (HeavenlyStem::Canh, HeavenlyStem::Binh),
        (HeavenlyStem::Binh, HeavenlyStem::Mau),
        (HeavenlyStem::Tan, HeavenlyStem::Mau),
        (HeavenlyStem::Dinh, HeavenlyStem::Canh),
        (HeavenlyStem::Nham, HeavenlyStem::Canh),
        (HeavenlyStem::Mau, HeavenlyStem::Nham),
        (HeavenlyStem::Quy, HeavenlyStem::Nham),
    ];
    for &(day_stem, ty_stem) in table {
        let cc = hour_can_chi(day_stem, 23).unwrap(); // 23h luôn là Tý
        assert_eq!(cc.branch, EarthlyBranch::Ty);
        assert_eq!(cc.stem, ty_stem);
    }
}

// =========================================================================
// 9. Ngũ Hành của Can / Chi
// =========================================================================

#[test]
fn nguhanh_of_stem_complete_table() {
    use NguHanh::*;
    let table: &[(HeavenlyStem, NguHanh)] = &[
        (HeavenlyStem::Giap, Moc),
        (HeavenlyStem::At, Moc),
        (HeavenlyStem::Binh, Hoa),
        (HeavenlyStem::Dinh, Hoa),
        (HeavenlyStem::Mau, Tho),
        (HeavenlyStem::Ky, Tho),
        (HeavenlyStem::Canh, Kim),
        (HeavenlyStem::Tan, Kim),
        (HeavenlyStem::Nham, Thuy),
        (HeavenlyStem::Quy, Thuy),
    ];
    for &(stem, expected) in table {
        assert_eq!(nguhanh_of_stem(stem), expected);
    }
}

#[test]
fn nguhanh_of_branch_complete_table() {
    use NguHanh::*;
    let table: &[(EarthlyBranch, NguHanh)] = &[
        (EarthlyBranch::Ty, Thuy),
        (EarthlyBranch::Suu, Tho),
        (EarthlyBranch::Dan, Moc),
        (EarthlyBranch::Mao, Moc),
        (EarthlyBranch::Thin, Tho),
        (EarthlyBranch::Ty2, Hoa),
        (EarthlyBranch::Ngo, Hoa),
        (EarthlyBranch::Mui, Tho),
        (EarthlyBranch::Than, Kim),
        (EarthlyBranch::Dau, Kim),
        (EarthlyBranch::Tuat, Tho),
        (EarthlyBranch::Hoi, Thuy),
    ];
    for &(b, expected) in table {
        assert_eq!(
            nguhanh_of_branch(b),
            expected,
            "branch {:?} expected {:?}",
            b,
            expected
        );
    }
}

// =========================================================================
// 10. Roundtrip (S2L ↔ L2S) — 80 random date vectors
// =========================================================================

#[test]
fn roundtrip_solar_to_lunar_to_solar() {
    for &(d, m, y) in test_data::ROUNDTRIP_SOLAR {
        let solar = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap();
        let lunar = solar_to_lunar(solar).unwrap();
        let result = lunar_to_solar(lunar).unwrap();
        assert_eq!(
            result, solar,
            "Roundtrip {} (Dương) → {:?} (Âm) → {} phải khớp Dương gốc",
            solar, lunar, result
        );
    }
}

// =========================================================================
// 11. Lỗi ngoài phạm vi & invalid date
// =========================================================================

#[test]
fn out_of_range_solar_year_1899_returns_error() {
    let d = NaiveDate::from_ymd_opt(1899, 12, 31).unwrap();
    assert_eq!(
        solar_to_lunar(d).unwrap_err(),
        LunarError::OutOfRange("năm 1899 ngoài phạm vi hỗ trợ 1900–2100".into())
    );
}

#[test]
fn out_of_range_solar_year_2101_returns_error() {
    let d = NaiveDate::from_ymd_opt(2101, 1, 1).unwrap();
    assert!(matches!(
        solar_to_lunar(d).unwrap_err(),
        LunarError::OutOfRange(_)
    ));
}

#[test]
fn out_of_range_year_can_chi_returns_error() {
    assert!(matches!(
        year_can_chi(1899).unwrap_err(),
        LunarError::OutOfRange(_)
    ));
    assert!(matches!(
        year_can_chi(2101).unwrap_err(),
        LunarError::OutOfRange(_)
    ));
}

#[test]
fn invalid_lunar_month_13_returns_error() {
    let lunar = LunarDate {
        day: 1,
        month: 13,
        year: 2024,
        is_leap_month: false,
    };
    let err = lunar_to_solar(lunar).unwrap_err();
    assert!(matches!(err, LunarError::InvalidLunarDate(_)));
}

#[test]
fn invalid_leap_month_when_year_has_no_leap_returns_error() {
    // 2024 (Giáp Thìn) không có tháng nhuận → "1/1 nhuận 2024" là invalid.
    let lunar = LunarDate {
        day: 1,
        month: 1,
        year: 2024,
        is_leap_month: true,
    };
    let err = lunar_to_solar(lunar).unwrap_err();
    assert!(
        matches!(err, LunarError::InvalidLunarDate(_)),
        "expected InvalidLunarDate, got {:?}",
        err
    );
}

#[test]
fn invalid_day_31_returns_error() {
    let lunar = LunarDate {
        day: 31,
        month: 1,
        year: 2024,
        is_leap_month: false,
    };
    let err = lunar_to_solar(lunar).unwrap_err();
    assert!(matches!(err, LunarError::InvalidLunarDate(_)));
}

// =========================================================================
// 12. Edge cases — Dương lịch nhuận (29/2) + giao thừa (31/12 Dương có thể
//     thuộc năm âm cũ) + giờ Tý (23:00 hôm trước)
// =========================================================================

#[test]
fn leap_day_in_solar_leap_year_2024_feb29() {
    // 29/2/2024 Dương nhuận — phải convert được ra âm lịch bình thường.
    let d = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
    let lunar = solar_to_lunar(d).unwrap();
    // 2024-02-29 = ngày 20 tháng 1 Giáp Thìn (âm lịch)
    // Algorithm tính: should match...
    // L2S roundtrip:
    let rt = lunar_to_solar(lunar).unwrap();
    assert_eq!(rt, d);
}

#[test]
fn solar_dec_31_belongs_to_previous_lunar_year() {
    // 31/12/2023 Dương vẫn thuộc năm âm Quý Mão 2023 (Tết Giáp Thìn là 10/2/2024).
    // Specifically, nên là tháng 1 (lunar) âm lịch 2023 — không phải 2024.
    let d = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
    let lunar = solar_to_lunar(d).unwrap();
    assert!(
        lunar.year == 2023,
        "31/12/2023 must belong to lunar year 2023 (Quý Mão); got {}",
        lunar.year
    );
}

#[test]
fn hour_chi_wraps_at_23h_into_next_day_branch() {
    // 23h Dương lịch của bất kỳ ngày nào thuộc giờ Tý của ngày hôm sau theo trad tiếng Việt.
    // (theo传统的 "giờ Tý" begin 23h)
    let cc_23 = hour_can_chi(HeavenlyStem::Giap, 23).unwrap();
    let cc_0 = hour_can_chi(HeavenlyStem::Giap, 0).unwrap();
    assert_eq!(cc_23.branch, EarthlyBranch::Ty);
    assert_eq!(cc_0.branch, EarthlyBranch::Ty);
    assert_eq!(cc_23.stem, cc_0.stem, "giờ 23 và 0 thuộc cùng giờ Tý");
}

#[test]
fn hour_can_chi_out_of_range_returns_error() {
    assert!(matches!(
        hour_can_chi(HeavenlyStem::Giap, 24).unwrap_err(),
        LunarError::InvalidLunarDate(_)
    ));
}

// =========================================================================
// 13. BirthMoment struct — simple roundtrip
// =========================================================================

#[test]
fn birthmoment_basic_construction() {
    let bm = BirthMoment {
        solar_date: NaiveDate::from_ymd_opt(2024, 2, 10).unwrap(),
        hour: 10,
        minute: 30,
    };
    let lunar = solar_to_lunar(bm.solar_date).unwrap();
    assert_eq!(lunar.day, 1);
    assert_eq!(lunar.month, 1);
    assert_eq!(lunar.year, 2024);
}