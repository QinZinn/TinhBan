//! Tử Vi Đẩu Số — engine lập lá số (Bắc Tông).
//!
//! # Trường phái áp dụng
//!
//! Tuân theo **Bắc Tông** (trường phái phổ biến nhất tại Việt Nam hiện nay),
//! khác biệt nhỏ với Nam Phái ở một vài quy tắc an một số sao phụ (đặc biệt
//! Hỏa-Linh, Đào Hoa). Quy tắc an sao dùng trong module này được tham chiếu
//! trực tiếp từ `lasotuvi` (`doanguyen/lasotuvi` — mã nguồn mở của cùng cộng
//! đồng dùng thuật toán Hồ Ngọc Đức), đảm bảo đồng nhất trường phái với
//! `lich.vn`, `tuvi.cohoc.net`, phần lớn ứng dụng Tử Vi trực tuyến tại VN.
//!
//! Quy ước index trong module: dùng **`EarthlyBranch::index()` 0-based** (Tý=0,
//! Sửu=1, Dần=2, ..., Hợi=11) cho tất cả vị trí cung trên Địa bàn.
//!
//! # Phạm vi đã implement
//!
//! 1. **12 cung (Thập Nhị Cung)**: Mệnh, Phụ Mẫu, Phúc Đức, Điền Trạch, Quan
//!    Lộc, Nô Bộc, Thiên Di, Tật Ách, Tài Bạch, Tử Tức, Phu Thê, Huynh Đệ +
//!    Cung Thân. Cung Mệnh/Thân an theo tháng âm + giờ sinh (Mệnh lùi theo
//!    giờ, Thân tiến theo giờ) từ Dần.
//! 2. **Cục**: Tính (Kim/Mộc/Thủy/Hỏa/Thổ Cục) bằng nạp âm Can-Chi của tháng
//!    chứa cung Mệnh.
//! 3. **14 chính tinh**: Tử Vi + chuỗi Tử Vi tinh hệ (Liêm Trinh, Thiên Đồng,
//!    Vũ Khúc, Thái Dương, Thiên Cơ) + Thiên Phủ tinh hệ (Thái Âm, Tham Lang,
//!    Cự Môn, Thiên Tướng, Thiên Lương, Thất Sát, Phá Quân).
//! 4. **Phụ tinh**: Tả Phù / Hữu Bật (tháng), Văn Xương / Văn Khúc (giờ),
//!    Thiên Khôi / Thiên Việt (Can năm), Lộc Tồn (Can năm), Kình Dương / Đà La
//!    (kề 2 bên Lộc Tồn), Hỏa Tinh / Linh Tinh (Chi năm + giờ + âm-dương nam
//!    nữ), Thiên Mã (Chi năm), Đào Hoa (vòng Kiếp Sát → Đào Hoa).
//! 5. **Vòng Trường Sinh** (12 sao): an theo Cục, chiều `"Dương nam / Âm nữ
//!    thuận; Âm nam / Dương nữ nghịch"` (theo cụ Thiên Lương, sửa từ "Nam
//!    thuận Nữ nghịch" cũ).
//!
//! # Cố ý CHƯA implement ở giai đoạn 3 này
//!
//! - **Tứ Hóa** (Hóa Lộc / Hóa Quyền / Hóa Khoa / Hóa Kỵ).
//! - **Vòng Lưu Hà / Thiên Trù**, **vòng Thái Tuế**, **vòng Lộc Tồn mở rộng**
//!   (12 sao phụ của vòng Lộc Tồn: Lực Sĩ / Thanh Long / Tiểu Hao / ...).
//! - **Vòng Tuần / Triệt** (Triệt không / Tuần không).
//! - **Bộ hang trăm sao phụ**: Long Trì / Phượng Các / Tam Thai / Bát Tọa /
//!   Ân Quang / Thiên Quý / Cô Thần / Quả Tú / Thiên Hình / Thiên Riêu / Kiếp
//!   Sát / Hoa Cái / Địa Không / Địa Kiếp / Hồng Loan / Thiên Hỷ / Thiên Quan /
//!   Thiên Phúc / Thai Phụ / Phong Cáo / Đẩu Quân / Lưu Hà / Thiên Trù.
//!
//! Module này cung cấp đủ để một lá số "dùng được" cơ bản — 14 chính tinh đủ
//! để soi Mệnh / 3 hợp / cung chính, phụ tinh đủ để luận Lộc / Kỵ / Khôi
//! Việt / Hỏa Linh / Kình Đà.
//!
//! # Nguồn tham chiếu
//!
//! Đối chiếu với `doanguyen/lasotuvi` (Python — tham chiếu chính), cùng với
//! `lich.vn` / `tuvi.cohoc.net` (tham chiếu Web cho lá số mẫu).
//! Xem `tests/tuvi_reference.rs` cho 5 lá số mẫu đối chiếu chi tiết.

pub mod cuc;
pub mod palaces;
pub mod stars;
pub mod truong_sinh;
pub mod types;

pub use cuc::{Cuc, CucInfo};
pub use palaces::{dich_cung, Palace, PalaceName};
pub use stars::{Sao, SaoCategory};
pub use truong_sinh::{TruongSinhState, truong_sinh_positions};
pub use types::{Gender, TuViChart, TuViError};

use crate::{BirthMoment, EarthlyBranch, HeavenlyStem, LunarError};

/// Lập lá số Tử Vi Đẩu Số từ ngày giờ sinh + giới tính.
///
/// Trả về [`TuViChart`] chứa: 12 cung + 14 chính tinh + phụ tinh (Tả Hữu / Văn
/// / Khôi Việt / Lộc / Kình Đà / Hỏa Linh / Mã / Đào) + vòng Trường Sinh +
/// Cục. Lỗi: [`TuViError::OutOfRange`] nếu năm ngoài 1900–2100,
/// [`TuViError::Lunar`] nếu lịch âm không hợp lệ.
pub fn lap_la_so(birth: BirthMoment, gender: Gender) -> Result<TuViChart, TuViError> {
    let lunar = crate::solar_to_lunar(birth.solar_date)?;
    let birth_hour_chi = hour_branch_index(birth.hour);

    // 1. An cung Mệnh & Cung Thân (theo lasotuvi `cungChu`):
    //    cungMenh = dichCung(Dần, (m-1) - (h-1))         (lùi theo giờ)
    //    cungThan = dichCung(Dần, (m-1) + (h-1))         (tiến theo giờ)
    //    où m, h 0-based đều chạy, Dần = index 2.
    let month_idx = (lunar.month - 1) as i64;
    let hour_idx = birth_hour_chi as i64;
    let dan_0 = EarthlyBranch::Dan.index() as i64;
    let menh_idx = dich_cung_i64(dan_0, month_idx - hour_idx);
    let than_idx = dich_cung_i64(dan_0, month_idx + hour_idx);
    let menh_branch = EarthlyBranch::from_index(menh_idx as u8).unwrap();
    let than_branch = EarthlyBranch::from_index(than_idx as u8).unwrap();

    // 2. Cục (nạp âm Can-Chi của tháng âm chứa cung Mệnh).
    let year_stem_idx = can_of_year_idx(lunar.year);
    let year_stem = HeavenlyStem::from_index(year_stem_idx).unwrap();
    let cuc = cuc::tinh_cuc(menh_branch, year_stem);

    // 3. An 11 cung còn lại từ Mệnh theo thứ tự cố định (Mệnh → Phụ Mẫu (+1) →
    //    Phúc Đức (+2) → Điền Trạch (+3) → Quan Lộc (+4) → Nô Bộc (+5) →
    //    Thiên Di (+6 = đối cung) → Tật Ách (+7) → Tài Bạch (+8) → Tử Tức (+9)
    //    → Phu Thê (+10) → Huynh Đệ (+11)). Chiều thuận / nghịch theo giới
    //    tính × năm chỉ ảnh hưởng ĐẠI / TIỂU HẠN, không thay đổi vị trí 12
    //    cung trên Địa bàn.
    let palace_names = PalaceName::all_in_order();
    let mut palaces = Vec::with_capacity(12);
    for (i, name) in palace_names.iter().enumerate() {
        let branch_idx = dich_cung_i64(menh_idx as i64, i as i64) as u8;
        let branch = EarthlyBranch::from_index(branch_idx).unwrap();
        let mut palace = Palace::new(*name, branch);
        if *name == PalaceName::Menh {
            palace.is_menh = true;
        }
        if branch == than_branch {
            palace.is_than = true;
        }
        palaces.push(palace);
    }
    let palaces: [Palace; 12] = palaces
        .try_into()
        .map_err(|_| TuViError::Internal("palaces vector length != 12".into()))?;

    let mut chart = TuViChart {
        birth,
        gender,
        lunar,
        cuc,
        palaces,
        menh_branch,
        than_branch,
    };

    // 4. An 14 chính tinh + phụ tinh.
    stars::chinh_tinh::an_chinh_tinh(&mut chart)?;
    stars::phu_tinh::an_phu_tinh(&mut chart)?;

    // 5. Vòng Trường Sinh — chiều: Dương nam / Âm nữ thuận, Âm nam / Dương nữ
    //    nghịch. (`amDuongNamNu = gioiTinh * amDuongNamSinh`; +1 thuận, -1
    //    nghịch.)
    let am_duong_year = am_duong_of_year_chi(chart.lunar.year);
    let reverse = (gender.sign_i64() * am_duong_year) < 0;
    let ts = truong_sinh::truong_sinh_positions(chart.cuc.so, menh_branch, reverse);
    for (state, branch) in ts {
        if let Some(p) = chart.palace_mut(branch) {
            p.truong_sinh.push(state);
        }
    }

    Ok(chart)
}

/// Index của chi giờ ứng với `hour` (0..23): `((hour + 1) / 2) % 12`.
/// Giờ Tý = 23 (hôm trước) hoặc 0 → index 0; giờ Sửu = 1, 2 → index 1; ...
fn hour_branch_index(hour: u8) -> u8 {
    ((hour as i64 + 1) / 2).rem_euclid(12) as u8
}

/// Index Can của năm Dương lịch: `(year + 6) % 10` (Giáp=0, ..., Quý=9).
fn can_of_year_idx(year: i32) -> u8 {
    (year + 6).rem_euclid(10) as u8
}

/// Âm/Dương của Can năm: Giáp(0)/Bính(2)/Mậu(4)/Canh(6)/Nhâm(8) = Dương,
/// còn lại = Âm (theo amDuong trong `lasotuvi.AmDuong.thienCan`).
fn am_duong_of_year_chi(year: i32) -> i64 {
    let chi_i = (year + 8).rem_euclid(12) as i64;
    if chi_i % 2 == 0 {
        1
    } else {
        -1
    }
}

/// `dichCung` theo quy ước Ho (tất cả index 0-based). `base + offset mod 12`.
fn dich_cung_i64(base: i64, offset: i64) -> i8 {
    ((base + offset).rem_euclid(12)) as i8
}

impl From<LunarError> for TuViError {
    fn from(e: LunarError) -> Self {
        Self::Lunar(e)
    }
}

impl TuViChart {
    /// Trả mutable Palace theo Địa Chi.
    pub fn palace_mut(&mut self, branch: EarthlyBranch) -> Option<&mut Palace> {
        self.palaces.iter_mut().find(|p| p.branch == branch)
    }

    /// Trả immutable Palace theo Địa Chi.
    pub fn palace(&self, branch: EarthlyBranch) -> Option<&Palace> {
        self.palaces.iter().find(|p| p.branch == branch)
    }

    /// Trả cung Mệnh.
    pub fn menh(&self) -> &Palace {
        self.palace(self.menh_branch).expect("menh_branch valid")
    }

    /// Trả cung Thân.
    pub fn than(&self) -> &Palace {
        self.palace(self.than_branch).expect("than_branch valid")
    }
}

#[cfg(test)]
mod smoke {
    use super::*;

    #[test]
    fn lap_la_so_smoke_does_not_panic() {
        let birth = BirthMoment {
            solar_date: chrono::NaiveDate::from_ymd_opt(1991, 10, 24).unwrap(),
            hour: 7,
            minute: 30,
        };
        let chart = lap_la_so(birth, Gender::Nam).unwrap();
        assert_eq!(chart.palaces.len(), 12);
        // 14 chính tinh phải đều mặt sau khi an.
        let tuvi_count = chart
            .palaces
            .iter()
            .map(|p| {
                p.stars
                    .iter()
                    .filter(|s| matches!(s.category(), SaoCategory::ChinhTinh))
                    .count()
            })
            .sum::<usize>();
        assert_eq!(tuvi_count, 14, "expected 14 chính tinh");
        // Cung Thân phải đúng 1 cung mark is_than=true.
        assert_eq!(chart.palaces.iter().filter(|p| p.is_than).count(), 1);
        // Cung Mệnh phải đúng 1 cung mark is_menh=true.
        assert_eq!(chart.palaces.iter().filter(|p| p.is_menh).count(), 1);
    }
}