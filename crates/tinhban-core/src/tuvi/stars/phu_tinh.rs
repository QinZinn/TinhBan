//! An phụ tinh phổ biến (Tả Phù / Hữu Bật / Văn Xương / Văn Khúc / Thiên Khôi /
//! Thiên Việt / Lộc Tồn / Kình Dương / Đà La / Hỏa Tinh / Linh Tinh / Thiên
//! Mã / Đào Hoa).
//!
//! Tất cả quy tắc在本 module tham chiếu `lasotuvi.App::lapDiaBan` (Python), đã
//! đối chiếu với source 1-based của Ho (`doanguyen/lasotuvi`). Port dùng quy
//! ước 0-based (`EarthlyBranch::index()` 0..11, `HeavenlyStem::index()` 0..9)
//! cho tất cả index.
//!
//! # Bảng quy tắc tham chiếu nhanh
//!
//! | Sao             | Yếu tố an                       | Vị trí                                  |
//! |-----------------|---------------------------------|-----------------------------------------|
//! | Tả Phù          | tháng sinh                      | dichCung(Thìn, m-1)                     |
//! | Hữu Bật         | đối xứng Tả Phù qua trục Sửu-Mùi| (2 - ta_phu_1based) → 0-based           |
//! | Văn Xương       | giờ sinh (chi giờ)              | dichCung(Thìn, gi-1)                    |
//! | Văn Khúc        | đối xứng Văn Xương qua Sửu-Mùi  | (2 - vx_1based)                          |
//! | Thiên Khôi      | Can năm sinh                    | bảng                                    |
//! | Thiên Việt      | đối xứng Thiên Khôi qua Sửu-Mùi | (4 - khoi_0)-based                       |
//! | Lộc Tồn         | Can năm sinh                    | bảng (theo vitriDiaBan của lasotuvi)    |
//! | Kình Dương      | 1 cung sau Lộc Tồn (thuận)      | dichCung(LocTon, +1)                     |
//! | Đà La           | 1 cung trước Lộc Tồn (nghịch)   | dichCung(LocTon, -1)                     |
//! | Hỏa Tinh        | Chi năm + giờ + g×âm_dương năm  | thuật toán riêng (lasotuvi `timHoaLinh`)|
//! | Linh Tinh        | như Hỏa Tinh, cặp kề            | thuật toán riêng                         |
//! | Thiên Mã         | Chi năm (tam hợp cục)           | bảng: Dần-Hợi-Tỵ-Thân cycle             |
//! | Đào Hoa          | vòng Kiếp Sát (Mã+3+4)          | derived from Mã                          |
//!
//! **Lưu ý**: quy ước "đối xứng qua trục Sửu-Mùi" trong lasotuvi dùng công
//! thức `dichCung(2, 2 - x)` 1-based (Sửu = 2 1-based). Port 0-based:
//! `(1 - x_0based).rem_euclid(12)` không đúng — thực chất `dichCung(2, 2 - x)`
//! 1-based = (1 + (1 - (x-1))).rem_euclid(12) = (2 - x) rem 12 → port 0-based:
//! `(1 - x_0) rem 12`. Cụ thể: công thức Ho `dichCung(2, 2 - x)` 1-based
//! nhưng trong Ho `dichCung` chỉ cộng vào mod 12. Nếu x 1-based, `2 + 2 - x =
//! 4 - x`, port 0-based → `(3 - x_0) rem 12`. OK để an toàn, sẵn出自 từ Ho
//! code: `dichCung(2, 2 - viTriTaPhu)`: đây 2 là Sửu 1-based, công thức thực
//! sự посчитать được bằng: `((1) + (2 - viTriTaPhu_1based)).rem_euclid(12)`,
//! cũng chính `(3 - viTriTaPhu_1based).rem_euclid(12)` ≡ `(3 - (ta_phu_0+1))
//! rem 12` = `(2 - ta_phu_0) rem 12`. Dùng công thức này cho 0-based.

use crate::{EarthlyBranch, HeavenlyStem};
use crate::tuvi::dich_cung;
use crate::tuvi::stars::Sao;
use crate::tuvi::types::{Gender, TuViChart, TuViError};

/// Helper: trả chi-giờ index (0..11) cho `hour` 0..23.
pub(super) fn hour_branch_index_pub(hour: u8) -> u8 {
    (((hour as i64) + 1) / 2).rem_euclid(12) as u8
}

/// Entry point: an toàn bộ phụ tinh đã implement vào `chart.palaces`.
pub fn an_phu_tinh(chart: &mut TuViChart) -> Result<(), TuViError> {
    let month_idx = (chart.lunar.month - 1) as i64; // 0-based 0..11
    let hour_idx = hour_branch_index_pub(chart.birth.hour) as i64;
    let year_stem_idx = ((chart.lunar.year as i64 + 6).rem_euclid(10)) as u8; // 0..9 (Giáp=0)
    let year_branch_idx = ((chart.lunar.year as i64 + 8).rem_euclid(12)) as u8; // 0..11 (Tý=0)
    let year_stem = HeavenlyStem::from_index(year_stem_idx).unwrap();

    // === Tả Phù, Hữu Bật (theo tháng sinh) ===
    // Ho: TaPhu = dichCung(Thìn=5_1based, thang-1)
    // 0-based: Thìn = index 4. TaPhu = dichCung(Thìn, m-1) = (4 + month_idx) % 12.
    let ta_phu_idx = (4 + month_idx).rem_euclid(12);
    let ta_phu = EarthlyBranch::from_index(ta_phu_idx as u8).unwrap();
    add_at(chart, ta_phu, Sao::TaPhu);
    // HuuBat = dichCung(Sửu=2_1based, 2 - TaPhu_1based)
    // 0-based: HuuBat = (2 - ta_phu_idx).rem_euclid(12) — theo nhận ở doc༘
    let huu_bat_idx = (2 - ta_phu_idx).rem_euclid(12);
    add_at(chart, EarthlyBranch::from_index(huu_bat_idx as u8).unwrap(), Sao::HuuBat);

    // === Văn Xương, Văn Khúc (theo giờ sinh - chi giờ) ===
    // Ho: VanKhuc = dichCung(Thìn=5_1based, gioSinh-1)  (NOT VanXuong!)
    //    VanXuong = dichCung(Sửu=2_1based, 2 - viTriVanKhuc_1based)
    // (Phiên bản đầu tôi nhầm đổi giữa Văn Xương vs Văn Khúc: Ho an Van KHÚC
    // tại Thìn chiều thuận theo giờ, Văn XƯƠNG đối xứng qua trục Sửu-Mùi.)
    let gio_1 = hour_idx + 1; // lasotuvi's 1-based chi-giờ index
    let van_khuc_1based = dich_cung_1(5, gio_1 - 1);
    let van_khuc_idx = (van_khuc_1based - 1).rem_euclid(12);
    add_at(chart, EarthlyBranch::from_index(van_khuc_idx as u8).unwrap(), Sao::VanKhuc);
    // VanXuong 0-based = (3 - VanKhuc_1based) rem 12 — theo deriv ở doc.
    let van_xuong_idx = (3 - van_khuc_1based).rem_euclid(12);
    add_at(chart, EarthlyBranch::from_index(van_xuong_idx as u8).unwrap(), Sao::VanXuong);

    // === Thiên Khôi, Thiên Việt (theo Can năm sinh) ===
    // Ho: khoiViet = [None, 2, 1, 12, 10, 8, 1, 8, 7, 6, 4] (index 1..10)
    // Trong đó Can năm: Giáp=1, Ất=2, ..., Quý=10.
    let year_stem_1 = year_stem_idx as usize + 1; // 1..10
    let khoi_viet_table = [0u8, 2, 1, 12, 10, 8, 1, 8, 7, 6, 4]; // index 0 unused, 1..10 used
    let khoi_1 = khoi_viet_table[year_stem_1];
    let khoi_0 = (khoi_1 as i64 - 1).rem_euclid(12) as u8; // port 0-based
    add_at(chart, EarthlyBranch::from_index(khoi_0).unwrap(), Sao::ThienKhoi);
    // Ho: viTriThienViet = dichCung(Thìn=5_1based, 5 - viTriThienKhoi_1based)
    // ⇒ 1-based: (5 + (5 - khoi_1)) = (10 - khoi_1) mod 12 normalized
    // ⇒ 0-based: (9 - khoi_1) rem 12 (subtract 1 from 1-based)
    let viet_0 = (9 - khoi_1 as i64).rem_euclid(12) as u8;
    add_at(chart, EarthlyBranch::from_index(viet_0).unwrap(), Sao::ThienViet);

    // === Lộc Tồn (theo Can năm sinh) ===
    // Ho: viTriLocTon = thienCan[canNam]['vitriDiaBan'] (1..12)
    // Bảng (index 1..10, Giáp=1...Quý=10):
    //   Giáp(1)=3 (Dần), Ất(2)=4 (Mão), Bính(3)=6 (Tỵ), Đinh(4)=7 (Ngọ),
    //   Mậu(5)=6 (Tỵ), Kỷ(6)=7 (Ngọ), Canh(7)=9 (Thân), Tân(8)=10 (Dậu),
    //   Nhâm(9)=12 (Hợi), Quý(10)=1 (Tý).
    // Port 0-based:
    let loc_ton_1 = match year_stem {
        HeavenlyStem::Giap => 3,
        HeavenlyStem::At => 4,
        HeavenlyStem::Binh => 6,
        HeavenlyStem::Dinh => 7,
        HeavenlyStem::Mau => 6,
        HeavenlyStem::Ky => 7,
        HeavenlyStem::Canh => 9,
        HeavenlyStem::Tan => 10,
        HeavenlyStem::Nham => 12,
        HeavenlyStem::Quy => 1,
    };
    let loc_ton_0 = (loc_ton_1 as i64 - 1).rem_euclid(12) as u8;
    let loc_ton = EarthlyBranch::from_index(loc_ton_0).unwrap();
    add_at(chart, loc_ton, Sao::LocTon);

    // === Kình Dương, Đà La (kề Lộc Tồn) ===
    // Ho: DaLa = dichCung(LocTon, -1); KinhDuong = dichCung(LocTon, +1)
    let da_la = dich_cung(loc_ton, -1);
    let kinh_duong = dich_cung(loc_ton, 1);
    add_at(chart, da_la, Sao::DaLa);
    add_at(chart, kinh_duong, Sao::KinhDuong);

    // === Hỏa Tinh, Linh Tinh (theo Chi năm + giờ + giới tính × âm-dương năm) ===
    // Port `lasotuvi.AmDuong.timHoaLinh` (chiNam, gioSinh, gioiTinh, amDuongNamSinh).
    // Ho dùng: chiNam, gioSinh, gioiTinh (1/-1), amDuongNamSinh (1/-1 từ Can năm).
    // amDuongNamSinh ( theo Can): Giáp/Bính/Mậu/Canh/Nhâm = +1; còn lại = -1
    let am_duong_nam_sinh = match year_stem {
        HeavenlyStem::Giap | HeavenlyStem::Binh | HeavenlyStem::Mau
        | HeavenlyStem::Canh | HeavenlyStem::Nham => 1i64,
        HeavenlyStem::At | HeavenlyStem::Dinh | HeavenlyStem::Ky
        | HeavenlyStem::Tan | HeavenlyStem::Quy => -1,
    };
    let (hoa_tinh, linh_tinh) = tim_hoa_linh(
        // lasotuvi expects chiNam 1-based: pass year_branch_idx (0-based) + 1.
        (year_branch_idx as i64) + 1,
        hour_idx + 1, // Ho's `gioSinh` is 1-based (1..12)
        chart.gender,
        am_duong_nam_sinh,
    );
    add_at(chart, hoa_tinh, Sao::HoaTinh);
    add_at(chart, linh_tinh, Sao::LinhTinh);

    // === Thiên Mã (theo Chi năm, tam hợp cục) ===
    // Ho: timThienMa(chiNam) — 1-based chiNam; port convert.
    // Bảng Ho:
    //   demNghich = chiNam % 4
    //   1 → Dần(3_1based); 2 → Hợi(12); 3 → Thân(9); 0 → Tỵ(6).
    // Port 0-based:
    let chi_year_1 = (year_branch_idx as i64) + 1;
    let dem_nghich = chi_year_1.rem_euclid(4);
    let thien_ma_1 = match dem_nghich {
        1 => 3i64,  // Dần
        2 => 12,    // Hợi
        3 => 9,     // Thân
        0 => 6,     // Tỵ
        _ => unreachable!(),
    };
    let thien_ma_0 = (thien_ma_1 as i64 - 1).rem_euclid(12) as u8;
    add_at(chart, EarthlyBranch::from_index(thien_ma_0).unwrap(), Sao::ThienMa);

    // === Đào Hoa được an theo vòng Kiếp Sát (lasotuvi: KiepSat → DaoHoa) ===
    // Ho: viTriKiepSat = dichCung(ThienMa, +3); viTriDaoHoa = dichCung(KiepSat, +4)
    let thien_ma_branch = EarthlyBranch::from_index(thien_ma_0).unwrap();
    let kiep_sat_branch = dich_cung(thien_ma_branch, 3);
    let dao_hoa_branch = dich_cung(kiep_sat_branch, 4);
    add_at(chart, dao_hoa_branch, Sao::DaoHoa);

    Ok(())
}

/// Port `lasotuvi.AmDuong.timHoaLinh(chiNam, gioSinh, gioiTinh, amDuongNamSinh)`.
///
/// Lưu ý Ho dùng `chiNam` 1..12, `gioSinh` 1..12. Trả tuple `(viTriHoaTinh,
/// viTriLinhTinh)` (vị trí 0-based EarthlyBranch).
fn tim_hoa_linh(chi_year_1: i64, gio_1: i64, gender: Gender, am_duong_year: i64) -> (EarthlyBranch, EarthlyBranch) {
    // Khởi cung dựa trên tam hợp cục (theo Ho `timHoaLinh`):
    //   [3, 7, 11] (Dần-Ngọ-Tuất) → khoiHoa=2 (Sửu), khoiLinh=4 (Mão)
    //   [1, 5, 9]  (Tý-Thìn-Thân) → khoiHoa=3 (Dần), khoiLinh=11 (Hợi)
    //   [6, 10, 2] (Tỵ-Dậu-Sửu)    → khoiHoa=11 (Hợi), khoiLinh=4 (Mão)
    //   [12, 4, 8] (Hợi-Mão-Mùi)   → khoiHoa=10 (Tuất), khoiLinh=11 (Hợi)
    let (khoi_hoa_1, khoi_linh_1);
    if [3, 7, 11].contains(&chi_year_1) {
        khoi_hoa_1 = 2; khoi_linh_1 = 4;
    } else if [1, 5, 9].contains(&chi_year_1) {
        khoi_hoa_1 = 3; khoi_linh_1 = 11;
    } else if [6, 10, 2].contains(&chi_year_1) {
        khoi_hoa_1 = 11; khoi_linh_1 = 4;
    } else if [12, 4, 8].contains(&chi_year_1) {
        khoi_hoa_1 = 10; khoi_linh_1 = 11;
    } else {
        unreachable!("chi_year out of tam hợp: {}", chi_year_1);
    }

    // Công thức Ho:
    //   if (gioiTinh * amDuongNamSinh) == -1: nghịch chiều
    //     viTriHoaTinh = dichCung(khoiHoa+1, -gioSinh)   (khoiHoa+1: dịch thêm 1 cung)
    //     viTriLinhTinh = dichCung(khoiLinh-1, +gioSinh)
    //   elif (... ) == 1: thuận chiều
    //     viTriHoaTinh = dichCung(khoiHoa-1, +gioSinh)
    //     viTriLinhTinh = dichCung(khoiLinh+1, -gioSinh)
    let sign = gender.sign_i64() * am_duong_year;
    let (hoa_1, linh_1);
    if sign == -1 {
        hoa_1 = dich_cung_1(khoi_hoa_1 + 1, -gio_1);
        linh_1 = dich_cung_1(khoi_linh_1 - 1, gio_1);
    } else if sign == 1 {
        hoa_1 = dich_cung_1(khoi_hoa_1 - 1, gio_1);
        linh_1 = dich_cung_1(khoi_linh_1 + 1, -gio_1);
    } else {
        unreachable!();
    }

    let hoa_0 = ((hoa_1 - 1).rem_euclid(12)) as u8;
    let linh_0 = ((linh_1 - 1).rem_euclid(12)) as u8;
    (
        EarthlyBranch::from_index(hoa_0).unwrap(),
        EarthlyBranch::from_index(linh_0).unwrap(),
    )
}

/// dichCung theo quy ước 1-based của Ho (mod 12, trả về 1..12).
/// `dichCung(cungBanDau, *args)` = sum tất cả mod 12, nếu 0 thì trả 12.
fn dich_cung_1(base: i64, offset: i64) -> i64 {
    let v = (base + offset).rem_euclid(12);
    if v == 0 {
        12
    } else {
        v
    }
}

/// Thêm `sao` vào cung có Địa Chi `branch` của `chart`.
fn add_at(chart: &mut TuViChart, branch: EarthlyBranch, sao: Sao) {
    if let Some(p) = chart.palace_mut(branch) {
        p.add_star(sao);
    }
}

// Expose `hour_branch_index` for sibling modules to use.
use hour_branch_index_pub as _hour_branch_index_pub;
// Silence dead_code if not called elsewhere.
#[allow(unused_imports)]
use _hour_branch_index_pub as _;