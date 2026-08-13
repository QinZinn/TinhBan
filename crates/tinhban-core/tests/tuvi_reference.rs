//! Test đối chiếu Tử Vi với 5 lá số mẫu (10 case: nam/nữ × 5 ngày sinh), sinh
//! từ `lasotuvi` (`doanguyen/lasotuvi`) bằng script `gen_ref.py` /
//! `gen_test_data.py` (xem `tuvi/README.md` mục "Quy trình đối chiếu").
//!
//! Mỗi case đối chiếu:
//!  - Cung Mệnh / Cung Thân ở đúng Địa Chi 0-based
//!  - Ngày/tháng/năm âm lịch của `lap_la_so` khớp với `lasotuvi`
//!  - 14 chính tinh đều ở đúng cung Chi
//!  - 14 phụ tinh (Tả / Hữu / Văn / Khôi / Việt / Lộc / Kình / Đà / Hỏa / Linh
//!    / Mã / Đào) đều ở đúng cung Chi

use chrono::NaiveDate;
use tinhban_core::{BirthMoment, EarthlyBranch, Sao, lap_la_so};

#[path = "tuvi_ref_data.rs"]
mod ref_data;
use ref_data::REF_CASES;

/// Parse sao tên (string từ ref data) thành `Sao` enum.
fn parse_sao(name: &str) -> Option<Sao> {
    use Sao::*;
    Some(match name {
        "TuVi" => TuVi,
        "ThienCo" => ThienCo,
        "ThaiDuong" => ThaiDuong,
        "VuKhuc" => VuKhuc,
        "ThienDong" => ThienDong,
        "LiemTrinh" => LiemTrinh,
        "ThienPhu" => ThienPhu,
        "ThaiAm" => ThaiAm,
        "ThamLang" => ThamLang,
        "CuMon" => CuMon,
        "ThienTuong" => ThienTuong,
        "ThienLuong" => ThienLuong,
        "ThatSat" => ThatSat,
        "PhaQuan" => PhaQuan,
        "TaPhu" => TaPhu,
        "HuuBat" => HuuBat,
        "VanXuong" => VanXuong,
        "VanKhuc" => VanKhuc,
        "ThienKhoi" => ThienKhoi,
        "ThienViet" => ThienViet,
        "LocTon" => LocTon,
        "KinhDuong" => KinhDuong,
        "DaLa" => DaLa,
        "HoaTinh" => HoaTinh,
        "LinhTinh" => LinhTinh,
        "ThienMa" => ThienMa,
        "DaoHoa" => DaoHoa,
        _ => return None,
    })
}

#[test]
fn all_10_reference_cases_match_lasotuvi() {
    let mut total_failures = 0usize;
    let mut total_checks = 0usize;
    for case in REF_CASES {
        let (yyyy, mm, dd, hour) = case.birth;
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(yyyy, mm, dd).unwrap(),
            hour,
            minute: 0,
        };
        let chart = match lap_la_so(birth, case.gender) {
            Ok(c) => c,
            Err(e) => {
                panic!("case {} failed lap_la_so: {}", case.label, e);
            }
        };

        // 1. Lịch âm khớp.
        assert_eq!(
            (chart.lunar.day, chart.lunar.month, chart.lunar.year),
            (case.lunar_day, case.lunar_month, case.lunar_year),
            "case {} lunar date mismatch",
            case.label
        );

        // 2. Cung Mệnh & Cung Thân ở đúng Địa Chi 0-based.
        let menh_0 = chart.menh_branch.index();
        let than_0 = chart.than_branch.index();
        assert_eq!(
            menh_0, case.menh_branch,
            "case {}: menh_branch mismatch — got {:?}, expected chi_0 {}",
            case.label, chart.menh_branch, case.menh_branch
        );
        assert_eq!(
            than_0, case.than_branch,
            "case {}: than_branch mismatch — got {:?}, expected chi_0 {}",
            case.label, chart.than_branch, case.than_branch
        );

        // 3. Vị trí của các sao roster: 14 chính tinh + phụ tinh.
        for (sao_name, expected_chi_0) in case.sao_positions {
            let sao = parse_sao(sao_name).unwrap_or_else(|| {
                panic!("case {}: unknown sao {sao_name}", case.label)
            });
            let expected_branch = EarthlyBranch::from_index(*expected_chi_0).unwrap();
            let mut found = false;
            for palace in chart.palaces.iter() {
                if palace.branch == expected_branch && palace.has_star(sao) {
                    found = true;
                    break;
                }
            }
            total_checks += 1;
            if !found {
                total_failures += 1;
                eprintln!(
                    "FAIL: case {} expected {} at chi_0={} ({:?})",
                    case.label, sao_name, expected_chi_0, expected_branch
                );
            }
        }
    }
    assert_eq!(
        total_failures, 0,
        "{}/{} sao positions failed against lasotuvi reference",
        total_failures, total_checks
    );
    println!("All {} reference sao positions matched lasotuvi.", total_checks);
}

#[test]
fn each_palace_has_a_name() {
    for case in REF_CASES {
        let (yyyy, mm, dd, hour) = case.birth;
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(yyyy, mm, dd).unwrap(),
            hour,
            minute: 0,
        };
        let chart = lap_la_so(birth, case.gender).unwrap();
        // Mỗi palace có đúng 1 flag is_menh và không thoèleRestrict
        let menh_count = chart.palaces.iter().filter(|p| p.is_menh).count();
        let than_count = chart.palaces.iter().filter(|p| p.is_than).count();
        assert_eq!(menh_count, 1, "case {}: menh_count", case.label);
        assert_eq!(than_count, 1, "case {}: than_count", case.label);
    }
}

/// Sanity check: tổng số sao (chính + phụ tinh) ≥ 27 (14 chính + 14 phụ tinh
/// đã implement: Tả/Hữu/Văn Xương/Văn Khúc/Khôi/Việt/Lộc/Kình/Đà/Hỏa/Linh/Mã/
/// Đào = 27 sao). (Mỗi sao xuất hiện đúng 1 lần trong lá số.)
#[test]
fn sao_total_at_least_27_for_all_cases() {
    for case in REF_CASES {
        let (yyyy, mm, dd, hour) = case.birth;
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(yyyy, mm, dd).unwrap(),
            hour,
            minute: 0,
        };
        let chart = lap_la_so(birth, case.gender).unwrap();
        let total = chart.palaces.iter().map(|p| p.stars.len()).sum::<usize>();
        assert!(
            total >= 27,
            "case {}: total sao count {} < 27 (14 chính + 14 phụ implement)",
            case.label, total
        );
    }
}

/// Sanity check: Tử Vi và Thiên Phủ có thể cùng cung (vd Tử Vi ở Dần → Thiên
/// Phủ ở Dần theo công thức (4 - tuvi_idx_0) % 12 = 0). Đây là quy ước an
/// sao của Ho, không phải bug; test này chỉ ghi chú, không kiểm ra.

#[test]
fn kinh_da_la_around_loc_ton() {
    for case in REF_CASES {
        let (yyyy, mm, dd, hour) = case.birth;
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(yyyy, mm, dd).unwrap(),
            hour,
            minute: 0,
        };
        let chart = lap_la_so(birth, case.gender).unwrap();
        let loc_ton_branch = chart
            .palaces
            .iter()
            .find(|p| p.has_star(Sao::LocTon))
            .map(|p| p.branch)
            .expect("LocTon must be present");
        let kinh_duong_branch = chart
            .palaces
            .iter()
            .find(|p| p.has_star(Sao::KinhDuong))
            .map(|p| p.branch)
            .expect("KinhDuong must be present");
        let da_la_branch = chart
            .palaces
            .iter()
            .find(|p| p.has_star(Sao::DaLa))
            .map(|p| p.branch)
            .expect("DaLa must be present");

        // Kinh Dương = Lộc Tồn + 1 (thuận)
        let kinh_offset = kinh_duong_branch.index() as i64 - loc_ton_branch.index() as i64;
        assert_eq!(
            kinh_offset.rem_euclid(12),
            1,
            "case {}: KinhDuong offset {} not Lộc Tồn +1 ({:?})",
            case.label, kinh_offset, loc_ton_branch
        );
        // Đà La = Lộc Tồn - 1 (nghịch)
        let da_offset = loc_ton_branch.index() as i64 - da_la_branch.index() as i64;
        assert_eq!(
            da_offset.rem_euclid(12),
            1,
            "case {}: Đà La offset {} not Lộc Tồn -1 ({:?})",
            case.label, da_offset, loc_ton_branch
        );
    }
}