//! Tinh Bàn — lõi âm lịch & Can Chi (lunar calendar engine).
//!
//! # Nguồn thuật toán
//!
//! Thuật toán chuyển Dương lịch ↔ Âm lịch Việt Nam dựa trên thuật toán của
//! Hồ Ngọc Đức (Ho Ngoc Duc), công bố công khai từ cuối thập niên 1990, dùng
//! tính toán thiên văn (vị trí Mặt Trời / thời điểm sóc — new moon) theo múi
//! giờ UTC+7 của Việt Nam. Đây là thuật toán mà `lich.vn` và phần lớn lịch âm
//! Việt Nam trực tuyến dùng làm nguồn; đã được kiểm chứng rộng rãi.
//!
//! Toán thiên văn tham chiếu sách "Astronomical Algorithms" của Jean Meeus
//! (1998). Các hằng số đã được đối chiếu với nhiều bản port public trên GitHub
//! (`doanguyen/lasotuvi/Lich_HND.py` — nguồn chính thức từ tác giả, cùng
//! `vanng822/ramlich`, `kunkka19xx/look/lunar/src/lib.rs`,
//! `J2TEAM/vibe.j2team.org/.../lunar.ts`).
//!
//! # Phạm vi hỗ trợ
//!
//! Năm Dương lịch **1900 → 2100**. Ngoài phạm vi này, hàm chuyển đổi trả
//! [`LunarError::OutOfRange`]. Trong phạm vi,.algorithm có thể lệch 1 ngày ở
//! vài mốc giao thỏa khi sóc xảy ra rất sát nửa đêm VN — xem mục "Giới hạn" trong
//! `tinhban-core/README.md`.
//!
//! # Mốc gốc Julian Day
//!
//! Tất cả calculations dùng JD nguyên (integer JD = trưa UTC của ngày Dương).
//! Hàm `jd_from_date` trả về Julian Day Number tại *trưa UTC* của ngày đó
//! (theo convention thiên văn, JD=0 ứng với trưa UTC 1/1/4713 TCN).

use chrono::{Datelike, NaiveDate};
use std::f64::consts::PI;

mod astronomy;
mod canchi;
mod error;
pub mod bat_tu;
pub mod trach_nhat;
pub mod tuvi;

pub use canchi::{EarthlyBranch, HeavenlyStem, NguHanh, NguHanhElement};
pub use error::LunarError;

pub use canchi::{can_chi_display, nguhanh_of_branch, nguhanh_of_stem};

// Tử Vi re-exports cho ergonomic API (tinhban-core::lap_la_so, TuViChart, ...).
pub use tuvi::{
    lap_la_so, Cuc, CucInfo, Gender, Palace, PalaceName, Sao, SaoCategory,
    TruongSinhState, TuViChart, TuViError,
};

// Trạch Nhật (giai đoạn 5) re-exports.
pub use trach_nhat::{
    danh_gia_khoang, danh_gia_ngay, DayAssessment, HoangDaoHacDao, HourRange, KiengKy,
    ThanSat, Truc, TrucRating,
};

// Bát Tự re-exports cho ergonomic API. Lưu ý `Gender` ở đây là cùng enum với
// `tuvi::Gender` (re-export từ `crate::tuvi::Gender`); không tái-export ở
// `pub use bat_tu::Gender` để tránh ambiguity.
pub use bat_tu::{
    lap_bat_tu, BatTuChart, BatTuError, NguHanhCount, Pillar, TenGod,
};

// ===========================================================================
// App constants (back-compat từ giai đoạn 1, dùng cho `tinhban-api`).
// ===========================================================================

/// Tên app hiển thị ra UI/log.
pub const APP_NAME: &str = "Tinh Bàn";

/// tên app, getter tiện (cũng từ giai đoạn 1).
pub fn app_name() -> &'static str {
    APP_NAME
}

/// Version lấy từ Cargo.toml (workspace.package.version).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ===========================================================================
// Public domain types
// ===========================================================================

/// Ngày âm lịch dạng có cấu trúc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LunarDate {
    /// Ngày âm (1–30).
    pub day: u8,
    /// Tháng âm (1–12). Nếu [`is_leap_month`](Self::is_leap_month) = true,
    /// đây là số tháng kép của (ví dụ `month = 4` cho "tháng 4 nhuận").
    pub month: u8,
    /// Năm âm (bằng năm Dương lịch khi Lunar New Year của năm âm rơi vào
    /// tháng 1/2 năm Dương đó).
    pub year: i32,
    /// `true` nếu đây là tháng nhuận (tháng kép thứ 13 của năm nhuận).
    pub is_leap_month: bool,
}

/// Cặp Can–Chi (Thiên Can × Địa Chi). Ví dụ năm 2024 → Giáp Thìn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CanChi {
    pub stem: HeavenlyStem,
    pub branch: EarthlyBranch,
}

/// Ngày/giờ sinh dương lịch đầy đủ — input chung cho các giai đoạn sau
/// (Tử Vi giai đoạn 3, Bát Tự giai đoạn 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BirthMoment {
    pub solar_date: NaiveDate,
    /// 0–23 (giờ đồng hồ 24h, giờ Dương lịch).
    pub hour: u8,
    /// 0–59.
    pub minute: u8,
}

// ===========================================================================
// Public API
// ===========================================================================

const MIN_SOLAR_YEAR: i32 = 1900;
const MAX_SOLAR_YEAR: i32 = 2100;

/// Chuyển Dương lịch → Âm lịch Việt Nam (UTC+7).
///
/// Trả [`LunarError::OutOfRange`] nếu `date` ngoài 1900–2100 Dương lịch.
pub fn solar_to_lunar(date: NaiveDate) -> Result<LunarDate, LunarError> {
    check_range(date.year())?;
    let (d, m, y) = (date.day() as i64, date.month() as i64, date.year() as i64);
    let res = astronomy::solar_to_lunar_raw(d, m, y, astronomy::VN_TZ);
    let year: i32 = res
        .year
        .try_into()
        .map_err(|_| LunarError::OutOfRange(format!("năm {} ngoài i32", res.year)))?;
    Ok(LunarDate {
        day: res.day as u8,
        month: res.month as u8,
        year,
        is_leap_month: res.leap,
    })
}

/// Chuyển Âm lịch → Dương lịch Việt Nam (UTC+7). Hỗ trợ tháng nhuận.
///
/// Trả [`LunarError::OutOfRange`] nếu `lunar.year` ngoài 1900–2100,
/// [`LunarError::InvalidLunarDate`] nếu `lunar.is_leap_month = true` nhưng năm
/// âm đó không có tháng nhuận ở số tháng đã nêu (hoặc số tháng > 12 / ngày >
/// 30).
pub fn lunar_to_solar(lunar: LunarDate) -> Result<NaiveDate, LunarError> {
    check_range(lunar.year)?;
    if lunar.month < 1 || lunar.month > 12 {
        return Err(LunarError::InvalidLunarDate(format!(
            "tháng âm phải nằm trong 1..=12, nhận được {}",
            lunar.month
        )));
    }
    if lunar.day < 1 || lunar.day > 30 {
        return Err(LunarError::InvalidLunarDate(format!(
            "ngày âm phải nằm trong 1..=30, nhận được {}",
            lunar.day
        )));
    }
    let jd = astronomy::lunar_to_solar_raw(
        lunar.day as i64,
        lunar.month as i64,
        lunar.year as i64,
        lunar.is_leap_month,
        astronomy::VN_TZ,
    );
    // Kiểm tra tính hợp lệ: nếu claimed leap mà năm không có tháng nhuận của
    // số đó, kết quả JD có thể trùng với cùng tháng phi-nhuận → lung's pace.
    // Phát hiện bằng solar_to_lunar roundtrip:
    let roundtrip = astronomy::solar_to_lunar_raw_jd(jd, astronomy::VN_TZ);
    if roundtrip.leap != lunar.is_leap_month || roundtrip.month != lunar.month as i64 {
        return Err(LunarError::InvalidLunarDate(format!(
            "ngày âm {}/{}/{}{} không tồn tại (năm {} không có tháng {} nhuận)",
            lunar.day, lunar.month, lunar.year,
            if lunar.is_leap_month { " (nhuận)" } else { "" },
            lunar.year, lunar.month,
        )));
    }
    jd_to_naive_date(jd)
}

/// Can–Chi của **năm** (lunar year). Trả [`LunarError::OutOfRange`] nếu ngoài
/// 1900–2100.
pub fn year_can_chi(year: i32) -> Result<CanChi, LunarError> {
    check_range(year)?;
    Ok(canchi::year_can_chi(year as i64))
}

/// Can–Chi của **tháng** âm (dựa vào Can của năm và số tháng âm, theo quy tắc
/// "Ngũ Thử Độn"). Trả [`LunarError::OutOfRange`] nếu năm ngoài 1900–2100.
pub fn month_can_chi(lunar_year_can: HeavenlyStem, lunar_month: u8) -> Result<CanChi, LunarError> {
    if !(1..=12).contains(&lunar_month) {
        return Err(LunarError::InvalidLunarDate(format!(
            "tháng âm phải nằm trong 1..=12, nhận được {}",
            lunar_month
        )));
    }
    Ok(canchi::month_can_chi(lunar_year_can, lunar_month))
}

/// Can–Chi của **ngày** dương lịch (chu kỳ 60 Giáp Tý liên tục, dùng Julian Day
/// Number gốc). Trả [`LunarError::OutOfRange`] nếu `date` ngoài 1900–2100.
pub fn day_can_chi(date: NaiveDate) -> Result<CanChi, LunarError> {
    check_range(date.year())?;
    let jd = astronomy::jd_from_date(date.day() as i64, date.month() as i64, date.year() as i64);
    Ok(canchi::day_can_chi(jd))
}

/// Can–Chi của **giờ** (12 giờ Địa Chi truyền thống, mỗi giờ = 2 tiếng đồng
/// hồ; giờ Tý từ 23h đêm hôm trước). Can của giờ phụ thuộc Can của ngày, theo
/// quy tắc "Ngũ Thử Độn Thời".
///
/// `hour` 0–23 (giờ Dương lịch). `hour = 23` thuộc giờ Tý của *ngày hôm sau*
/// (theo truyền thống Tý bắt đầu 23h).
pub fn hour_can_chi(day_can: HeavenlyStem, hour: u8) -> Result<CanChi, LunarError> {
    if hour > 23 {
        return Err(LunarError::InvalidLunarDate(format!(
            "hour phải 0..=23, nhận được {}",
            hour
        )));
    }
    Ok(canchi::hour_can_chi(day_can, hour))
}

// ===========================================================================
// Internal helpers
// ===========================================================================

fn check_range(year: i32) -> Result<(), LunarError> {
    if !(MIN_SOLAR_YEAR..=MAX_SOLAR_YEAR).contains(&year) {
        return Err(LunarError::OutOfRange(format!(
            "năm {} ngoài phạm vi hỗ trợ 1900–2100",
            year
        )));
    }
    Ok(())
}

fn jd_to_naive_date(jd: i64) -> Result<NaiveDate, LunarError> {
    let (d, m, y) = astronomy::jd_to_date(jd);
    let y32: i32 = y
        .try_into()
        .map_err(|_| LunarError::OutOfRange(format!("năm {} ngoài i32", y)))?;
    NaiveDate::from_ymd_opt(y32, m as u32, d as u32)
        .ok_or_else(|| LunarError::InvalidLunarDate(format!("không hợp lệ: jd {} → {}-{}-{}", jd, y, m, d)))
}

// Re-export `PI` to silence unused warning when not needed elsewhere.
#[allow(dead_code)]
const _: f64 = PI;