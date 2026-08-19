//! Test đối chiếu Bát Tự engine với reference data (7 cases: 5 lá số mẫu + 2
//! edge case gần Lập Xuân). Sinh từ Python implementation cùng dùng Ho's
//! algorithm — đảm bảo hai chì mode `tinhban-core` internally consistent.
//!
//! Mỗi case kiểm tra Tứ Trụ (4 Can-Chi) khớp với reference. Hidden Stems và
//! Thập Thần có test riêng trong `bat_tu/hidden_stems.rs` và
//! `bat_tu/thap_than.rs` (module-level unit-tests).

use chrono::NaiveDate;
use tinhban_core::{BirthMoment, Gender, HeavenlyStem, EarthlyBranch, lap_bat_tu};

#[path = "bat_tu_ref_data.rs"]
mod ref_data;
use ref_data::REF_BT_CASES;

#[test]
fn all_reference_bt_cases_match_can_chi_indices() {
    let mut total_failures = 0;
    let mut total_checks = 0;
    for case in REF_BT_CASES {
        let (yyyy, mm, dd, hour) = case.birth;
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(yyyy, mm, dd).unwrap(),
            hour,
            minute: 0,
        };
        let chart = match lap_bat_tu(birth, Gender::Nam) {
            Ok(c) => c,
            Err(e) => panic!("case {}: lap_bat_tu failed: {}", case.label, e),
        };

        // Tứ Trụ Can-Chi phải khớp reference indices.
        let expected_year_can = HeavenlyStem::from_index(case.year_can_idx).unwrap();
        let expected_year_chi = EarthlyBranch::from_index(case.year_chi_idx).unwrap();
        let expected_month_can = HeavenlyStem::from_index(case.month_can_idx).unwrap();
        let expected_month_chi = EarthlyBranch::from_index(case.month_chi_idx).unwrap();
        let expected_day_can = HeavenlyStem::from_index(case.day_can_idx).unwrap();
        let expected_day_chi = EarthlyBranch::from_index(case.day_chi_idx).unwrap();
        let expected_hour_can = HeavenlyStem::from_index(case.hour_can_idx).unwrap();
        let expected_hour_chi = EarthlyBranch::from_index(case.hour_chi_idx).unwrap();

        let pairs: [(&str, &tinhban_core::CanChi, HeavenlyStem, EarthlyBranch); 4] = [
            ("year", &chart.year_pillar.can_chi, expected_year_can, expected_year_chi),
            ("month", &chart.month_pillar.can_chi, expected_month_can, expected_month_chi),
            ("day", &chart.day_pillar.can_chi, expected_day_can, expected_day_chi),
            ("hour", &chart.hour_pillar.can_chi, expected_hour_can, expected_hour_chi),
        ];
        for (pillar_name, cc, exp_can, exp_chi) in pairs {
            total_checks += 2;
            if cc.stem != exp_can {
                total_failures += 1;
                eprintln!(
                    "FAIL case {} {}: Can got {:?}, expected {:?} (idx {})",
                    case.label, pillar_name, cc.stem, exp_can, case.year_can_idx
                );
            }
            if cc.branch != exp_chi {
                total_failures += 1;
                eprintln!(
                    "FAIL case {} {}: Chi got {:?}, expected {:?}",
                    case.label, pillar_name, cc.branch, exp_chi
                );
            }
        }

        // NguHanhCount total = 8 every chart.
        assert_eq!(
            chart.nguhanh_count.total(), 8,
            "case {}: nguhanh count total must be 8",
            case.label
        );
    }
    assert_eq!(
        total_failures, 0,
        "{}/{} Tứ Trụ Can/Chi không khớp reference",
        total_failures, total_checks
    );
    println!("All {} Tứ Trụ Can/Chi match reference.", total_checks);
}

#[test]
fn lap_xuan_pre_birth_year_returns_previous_bt_year() {
    // case4: 29/01/1990 → BT year = 1989 (pre-Lập Xuân 1990).
    let birth = BirthMoment {
        solar_date: NaiveDate::from_ymd_opt(1990, 1, 29).unwrap(),
        hour: 9,
        minute: 0,
    };
    let chart = lap_bat_tu(birth, Gender::Nam).unwrap();
    assert_eq!(
        chart.year_pillar.branch(),
        EarthlyBranch::Ty2,
        "30/01/1990 must have BT year=Tỵ (pre-Lập Xuân 1990)"
    );
}

#[test]
fn lap_xuan_edge_case_one_day_post_returns_next_year() {
    // case6: 06/02/1991 — 1-2 days after Ho's predicted Lập Xuân 1991 → BT year = 1991 Tân Mùi.
    let birth = BirthMoment {
        solar_date: NaiveDate::from_ymd_opt(1991, 2, 6).unwrap(),
        hour: 14,
        minute: 0,
    };
    let chart = lap_bat_tu(birth, Gender::Nam).unwrap();
    assert_eq!(
        chart.year_pillar.stem(),
        HeavenlyStem::Tan,
        "case6: BT year=1991, Can=Tân"
    );
    assert_eq!(chart.year_pillar.branch(), EarthlyBranch::Mui, "case6: BT Chi=Mùi");
}

#[test]
fn lap_xuan_edge_case_three_days_pre_returns_previous_bt_year() {
    // case7: 01/02/1990 — 3 days before Ho's predicted Lập Xuân 1990 → BT year = 1989 Kỷ Tỵ.
    let birth = BirthMoment {
        solar_date: NaiveDate::from_ymd_opt(1990, 2, 1).unwrap(),
        hour: 8,
        minute: 0,
    };
    let chart = lap_bat_tu(birth, Gender::Nam).unwrap();
    assert_eq!(
        chart.year_pillar.stem(),
        HeavenlyStem::Ky,
        "case7: BT year=1989 (pre-Lập Xuân 1990)"
    );
    assert_eq!(chart.year_pillar.branch(), EarthlyBranch::Ty2, "case7: BT Chi=Tỵ");
}

#[test]
fn nhat_chu_is_day_pillar_stem() {
    for case in REF_BT_CASES {
        let (yyyy, mm, dd, hour) = case.birth;
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(yyyy, mm, dd).unwrap(),
            hour,
            minute: 0,
        };
        let chart = lap_bat_tu(birth, Gender::Nam).unwrap();
        assert_eq!(
            chart.nhat_chu(),
            chart.day_pillar.can_chi.stem,
            "case {}: Nhật Chủ must = trụ Ngày Can",
            case.label
        );
    }
}

#[test]
fn day_pillar_ten_god_is_none() {
    for case in REF_BT_CASES {
        let (yyyy, mm, dd, hour) = case.birth;
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(yyyy, mm, dd).unwrap(),
            hour,
            minute: 0,
        };
        let chart = lap_bat_tu(birth, Gender::Nam).unwrap();
        assert!(
            chart.day_pillar.ten_god.is_none(),
            "case {}: trụ Ngày ten_god must be None (Nhật Chủ)",
            case.label
        );
    }
}

#[test]
fn non_day_pillars_have_ten_god() {
    for case in REF_BT_CASES {
        let (yyyy, mm, dd, hour) = case.birth;
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(yyyy, mm, dd).unwrap(),
            hour,
            minute: 0,
        };
        let chart = lap_bat_tu(birth, Gender::Nam).unwrap();
        assert!(
            chart.year_pillar.ten_god.is_some()
                && chart.month_pillar.ten_god.is_some()
                && chart.hour_pillar.ten_god.is_some(),
            "case {}: pillars Năm/Tháng/Giờ must have ten_god set",
            case.label
        );
    }
}

#[test]
fn hidden_stems_attached_to_each_pillar() {
    for case in REF_BT_CASES {
        let (yyyy, mm, dd, hour) = case.birth;
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(yyyy, mm, dd).unwrap(),
            hour,
            minute: 0,
        };
        let chart = lap_bat_tu(birth, Gender::Nam).unwrap();
        // Mỗi trụ phải có 1-3 hidden stems.
        for (i, p) in [chart.year_pillar, chart.month_pillar, chart.day_pillar, chart.hour_pillar]
            .iter()
            .enumerate()
        {
            assert!(
                !p.hidden_stems.is_empty(),
                "case {}: pillar[{}] phải có ≥ 1 hidden stem",
                case.label, i
            );
            assert!(
                p.hidden_stems.len() <= 3,
                "case {}: pillar[{}] không nên có > 3 hidden stems",
                case.label, i
            );
        }
    }
}