//! **Audit Bug #7** — hằng số epoch sai trong `sun_longitude_deg_at_local_midnight`.
//!
//! Bug được phát hiện khi làm giai đoạn 5 (Trạch Nhật) nhưng **có nguồn gốc từ
//! giai đoạn 4** (Bát Tự): hàm bị lỗi được viết ra và dùng lần đầu ở
//! `bat_tu/tiet_khi.rs`. File này là bằng chứng kiểm chứng được cho toàn bộ kết
//! luận audit, thay vì chỉ ghi trong README.
//!
//! # Phạm vi ảnh hưởng (xác định bằng call graph)
//!
//! ```text
//! sun_longitude_deg_at_local_midnight   ← BỊ LỖI
//!   └── bat_tu::tiet_khi::{find_tiet_khi_jd, sun_longitude_at_end_of_day}
//!         ├── lap_xuan_jd            → bat_tu::lap_bat_tu  (TRỤ NĂM)
//!         ├── tiet_khi_jds_of_bt_year→ bat_tu::lap_bat_tu  (TRỤ THÁNG)
//!         └── tiet_month_branch_index→ trach_nhat::truc    (12 Trực, giai đoạn 5)
//!
//! sun_longitude_at_noon                 ← ĐÚNG (epoch gộp sẵn -0.5)
//!   └── get_sun_longitude
//!         └── solar_to_lunar / lunar_to_solar   → Tử Vi, ngày Âm lịch
//! ```
//!
//! Hai nhánh **tách biệt hoàn toàn**: lịch Âm (và do đó Tử Vi) chưa bao giờ đi
//! qua hàm bị lỗi.
//!
//! # Chiều sai
//!
//! Epoch `2451545.5` (thay vì `2451545.0`) làm kinh độ Mặt Trời bị tính cho thời
//! điểm sớm hơn 0.5 ngày → giá trị trả về thấp hơn thực tế ~0.49° → mốc tiết khí
//! bị đẩy **TRỄ**. Đo trên 1900–2100 (2412 mốc): **50.0% mốc trễ đúng 1 ngày,
//! 50.0% không đổi, 0 mốc sớm hơn** — sai một chiều, không bao giờ ngược lại.
//!
//! Hệ quả: ngày sinh rơi **đúng vào ngày giao tiết thật** bị xếp nhầm vào kỳ
//! TRƯỚC đó. Vùng rủi ro vì vậy rộng đúng **1 ngày** cho mỗi mốc bị lệch.

use chrono::NaiveDate;
use tinhban_core::{lap_bat_tu, BirthMoment, EarthlyBranch, Gender, HeavenlyStem};
use tinhban_core::bat_tu::tiet_khi::{find_tiet_khi_jd, TIET_KHI_TABLE};

#[path = "bat_tu_boundary_ref_data.rs"]
mod boundary_data;
use boundary_data::BOUNDARY_CASES;

/// JD nguyên của một ngày Dương lịch (thuật toán Gregorian chuẩn).
fn jd(y: i64, m: i64, d: i64) -> i64 {
    let a = (14 - m) / 12;
    let yy = y + 4800 - a;
    let mm = m + 12 * a - 3;
    d + (153 * mm + 2) / 5 + 365 * yy + yy / 4 - yy / 100 + yy / 400 - 32045
}

/// Khoảng cách (ngày) từ `jd_birth` tới mốc tiết khí gần nhất, xét cả 3 năm
/// lân cận để không bỏ sót mốc đầu/cuối năm.
fn khoang_cach_toi_moc_tiet_gan_nhat(jd_birth: i64, year: i32) -> i64 {
    let mut best = i64::MAX;
    for y in [year - 1, year, year + 1] {
        for &(deg, _) in TIET_KHI_TABLE.iter() {
            let search_year = if deg == 285 { y + 1 } else { y };
            if let Some(t) = find_tiet_khi_jd(deg as f64, search_year) {
                let d = jd_birth - t;
                if d.abs() < best.abs() {
                    best = d;
                }
            }
        }
    }
    best
}

// ===========================================================================
// 1. Bát Tự gần biên tiết khí — 11 case mới
// ===========================================================================

/// Toàn bộ 11 case biên phải khớp giá trị suy từ Đài Thiên văn Hồng Kông.
///
/// **8/11 case này bị mã TRƯỚC khi sửa epoch tính sai** (5 sai trụ Năm, 3 sai
/// trụ Tháng); 3 case còn lại là đối chứng mà mã cũ vốn đã đúng — có mặt để
/// chắc rằng bản sửa không "chữa quá tay" đẩy ranh giới lệch sang chiều kia.
#[test]
fn bat_tu_gan_bien_tiet_khi_khop_dai_thien_van_hong_kong() {
    let mut loi = Vec::new();
    for c in BOUNDARY_CASES {
        let (y, m, d, h) = c.birth;
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(y, m, d).expect("ngày hợp lệ"),
            hour: h,
            minute: 0,
        };
        let chart = match lap_bat_tu(birth, Gender::Nam) {
            Ok(x) => x,
            Err(e) => {
                loi.push(format!("{}: lap_bat_tu lỗi: {e}", c.label));
                continue;
            }
        };
        let mut chk = |ten: &str, thuc: (u8, u8), mong: (u8, u8)| {
            if thuc != mong {
                loi.push(format!(
                    "{} ({}-{:02}-{:02}): trụ {ten} = ({}, {}) nhưng HKO cho ({}, {}) — {}",
                    c.label, y, m, d, thuc.0, thuc.1, mong.0, mong.1, c.note
                ));
            }
        };
        chk(
            "Năm",
            (chart.year_pillar.can_chi.stem.index(), chart.year_pillar.can_chi.branch.index()),
            (c.year_can, c.year_chi),
        );
        chk(
            "Tháng",
            (chart.month_pillar.can_chi.stem.index(), chart.month_pillar.can_chi.branch.index()),
            (c.month_can, c.month_chi),
        );
        chk(
            "Ngày",
            (chart.day_pillar.can_chi.stem.index(), chart.day_pillar.can_chi.branch.index()),
            (c.day_can, c.day_chi),
        );
        chk(
            "Giờ",
            (chart.hour_pillar.can_chi.stem.index(), chart.hour_pillar.can_chi.branch.index()),
            (c.hour_can, c.hour_chi),
        );
    }
    assert!(
        loi.is_empty(),
        "{} case biên sai:\n  - {}",
        loi.len(),
        loi.join("\n  - ")
    );
}

/// Bộ case biên phải thực sự **nhắm vào vùng rủi ro**: đa số nằm đúng trên mốc
/// tiết khí (khoảng cách 0 ngày). Nếu ai đó vô tình thay bằng ngày "an toàn",
/// test này đỏ và bộ case mất tác dụng trong im lặng.
#[test]
fn bo_case_bien_thuc_su_nam_tren_moc_tiet_khi() {
    let tren_moc = BOUNDARY_CASES
        .iter()
        .filter(|c| {
            let (y, m, d, _) = c.birth;
            khoang_cach_toi_moc_tiet_gan_nhat(jd(y as i64, m as i64, d as i64), y) == 0
        })
        .count();
    assert!(
        tren_moc >= 8,
        "chỉ {tren_moc}/{} case nằm đúng trên mốc tiết khí — bộ case đã mất tính \
         'gần biên', không còn bảo vệ được vùng rủi ro của Bug #7",
        BOUNDARY_CASES.len()
    );

    let tung_sai = BOUNDARY_CASES.iter().filter(|c| c.sai_truoc_khi_sua).count();
    assert_eq!(
        tung_sai, 8,
        "phải có đúng 8 case từng bị mã cũ tính sai (5 trụ Năm + 3 trụ Tháng)"
    );
}

// ===========================================================================
// 2. Đánh giá lại 7 case Bát Tự của giai đoạn 4
// ===========================================================================

/// **Kết luận audit cho 7 case cũ**: không case nào bị Bug #7 ảnh hưởng — nhưng
/// lý do khác nhau, và một case chỉ thoát nhờ may mắn.
///
/// Test này khoá lại khoảng cách từ mỗi ngày sinh mẫu tới mốc tiết khí gần nhất,
/// để khẳng định "vẫn xanh" là có căn cứ chứ không phải trùng hợp không kiểm
/// chứng được.
///
/// Đáng chú ý: **case5 (2000-05-05) nằm ĐÚNG trên mốc Lập Hạ** — vị trí rủi ro
/// cao nhất có thể. Nó đúng chỉ vì Lập Hạ 2000 tình cờ thuộc 50% số mốc mà bug
/// KHÔNG đẩy lệch. Đây chính là lý do phải bổ sung bộ case biên ở mục 1: bộ test
/// cũ không hề "miễn nhiễm theo thiết kế".
#[test]
fn bay_case_giai_doan_4_ngoai_vung_rui_ro_nhung_case5_chi_thoat_nho_may() {
    // (nhãn, năm, tháng, ngày, khoảng cách kỳ vọng tới mốc tiết gần nhất)
    let cases = [
        ("case1", 1991, 10, 24, 15i64),
        ("case2", 2026, 2, 17, 13),
        ("case3", 2024, 2, 10, 6),
        ("case4", 1990, 1, 29, -6),
        ("case5", 2000, 5, 5, 0), // ← ĐÚNG trên mốc Lập Hạ
        ("case6", 1991, 2, 6, 2),
        ("case7", 1990, 2, 1, -3),
    ];
    for (label, y, m, d, mong_doi) in cases {
        let kc = khoang_cach_toi_moc_tiet_gan_nhat(jd(y, m, d), y as i32);
        assert_eq!(
            kc, mong_doi,
            "{label} ({y}-{m:02}-{d:02}): khoảng cách tới mốc tiết gần nhất phải là \
             {mong_doi} ngày"
        );
    }

    // Khẳng định tường minh điều khiến case5 thoát: mốc Lập Hạ 2000 KHÔNG bị bug
    // đẩy lệch. Ta kiểm bằng cách so mốc do code hiện tại tính với ngày thật
    // (5/5/2000 theo Đài Thiên văn Hồng Kông, đã quy về giờ VN).
    let lap_ha_2000 = find_tiet_khi_jd(45.0, 2000).expect("tìm được Lập Hạ 2000");
    assert_eq!(
        lap_ha_2000,
        jd(2000, 5, 5),
        "Lập Hạ 2000 phải rơi 5/5/2000 — nếu mốc này từng bị bug đẩy sang 6/5 thì \
         case5 của giai đoạn 4 đã sai ngay từ đầu"
    );
}

// ===========================================================================
// 3. Tử Vi không đi qua nhánh bị lỗi
// ===========================================================================

/// **Bằng chứng tường minh: Tử Vi Đẩu Số KHÔNG bị Bug #7 ảnh hưởng.**
///
/// Không kiểm bằng cách so kết quả (dễ lọt), mà kiểm thẳng **mã nguồn**: toàn bộ
/// module `tuvi/` không được nhắc tới `astronomy` hay `tiet_khi` dưới bất kỳ
/// hình thức nào. Tử Vi chỉ nhận đầu vào là ngày Âm lịch (`solar_to_lunar`) và
/// giờ sinh — mà `solar_to_lunar` đi qua `sun_longitude_at_noon`, nhánh có epoch
/// ĐÚNG.
///
/// Nếu sau này ai đó thêm phụ thuộc tiết khí vào Tử Vi (ví dụ để an sao theo
/// tiết), test này sẽ đỏ và buộc phải audit lại kết luận trên thay vì để nó âm
/// thầm hết đúng.
#[test]
fn tuvi_khong_phu_thuoc_astronomy_hay_tiet_khi() {
    let nguon: [(&str, &str); 8] = [
        ("tuvi/mod.rs", include_str!("../src/tuvi/mod.rs")),
        ("tuvi/types.rs", include_str!("../src/tuvi/types.rs")),
        ("tuvi/cuc.rs", include_str!("../src/tuvi/cuc.rs")),
        ("tuvi/palaces.rs", include_str!("../src/tuvi/palaces.rs")),
        ("tuvi/truong_sinh.rs", include_str!("../src/tuvi/truong_sinh.rs")),
        ("tuvi/stars/mod.rs", include_str!("../src/tuvi/stars/mod.rs")),
        ("tuvi/stars/chinh_tinh.rs", include_str!("../src/tuvi/stars/chinh_tinh.rs")),
        ("tuvi/stars/phu_tinh.rs", include_str!("../src/tuvi/stars/phu_tinh.rs")),
    ];
    for (ten, src) in nguon {
        // Bỏ dòng comment để không dính các đoạn chú thích có nhắc tên hàm.
        let code: String = src
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("//") && !t.starts_with("///") && !t.starts_with("//!")
            })
            .collect::<Vec<_>>()
            .join("\n");
        for cam in ["astronomy", "tiet_khi", "lap_xuan", "sun_longitude"] {
            assert!(
                !code.contains(cam),
                "{ten} nhắc tới {cam:?} — Tử Vi nay ĐÃ phụ thuộc nhánh từng dính \
                 Bug #7. Phải audit lại kết luận 'Tử Vi không bị ảnh hưởng' trong \
                 README trước khi bỏ assert này."
            );
        }
    }
}

/// Kiểm chứng bổ sung ở mức hành vi: hai ngày sinh nằm hai bên mốc Lập Xuân
/// nhưng **cùng ngày Âm lịch tháng/giờ khác nhau** không phải thứ Tử Vi quan
/// tâm — lá số Tử Vi chỉ đổi theo ngày/tháng Âm lịch và giờ sinh.
///
/// Cụ thể: 3/2/2024 và 4/2/2024 nằm hai bên mốc Lập Xuân 2024 (4/2). Với Bát Tự,
/// ranh giới này đổi cả trụ Năm. Với Tử Vi, hai ngày này chỉ khác nhau đúng ở
/// chỗ ngày Âm lịch tăng 1 — cung Mệnh (vốn chỉ phụ thuộc THÁNG Âm lịch và giờ)
/// phải giữ nguyên.
#[test]
fn tuvi_khong_doi_cung_menh_qua_moc_lap_xuan() {
    use tinhban_core::lap_la_so;
    let lam = |d: u32| {
        lap_la_so(
            BirthMoment {
                solar_date: NaiveDate::from_ymd_opt(2024, 2, d).unwrap(),
                hour: 10,
                minute: 0,
            },
            Gender::Nam,
        )
        .expect("lập được lá số")
    };
    let truoc = lam(3);
    let sau = lam(4);

    // Cùng tháng Âm lịch (24 và 25 tháng Chạp) + cùng giờ → cùng cung Mệnh.
    assert_eq!(truoc.lunar.month, sau.lunar.month, "hai ngày phải cùng tháng Âm");
    let menh_truoc = truoc.menh_branch;
    let menh_sau = sau.menh_branch;
    assert_eq!(
        menh_truoc, menh_sau,
        "cung Mệnh không được đổi khi bước qua mốc Lập Xuân — Tử Vi không dùng tiết khí"
    );

    // Để đối chiếu: cùng hai ngày đó, Bát Tự ĐỔI hẳn trụ Năm.
    let bt = |d: u32| {
        lap_bat_tu(
            BirthMoment {
                solar_date: NaiveDate::from_ymd_opt(2024, 2, d).unwrap(),
                hour: 10,
                minute: 0,
            },
            Gender::Nam,
        )
        .unwrap()
    };
    assert_eq!(bt(3).year_pillar.can_chi.stem, HeavenlyStem::Quy);
    assert_eq!(bt(3).year_pillar.can_chi.branch, EarthlyBranch::Mao);
    assert_eq!(bt(4).year_pillar.can_chi.stem, HeavenlyStem::Giap);
    assert_eq!(bt(4).year_pillar.can_chi.branch, EarthlyBranch::Thin);
}
