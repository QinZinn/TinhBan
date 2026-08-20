//! Cấu trúc dữ liệu công khai của module Tử Vi: `Gender`, `TuViError`,
//! `TuViChart` (re-export từ `palaces::Palace`).
//!
//! `TuViChart` và `Palace` lưu trữ kết quả của [`crate::tuvi::lap_la_so`].

use crate::{BirthMoment, EarthlyBranch, LunarDate, LunarError};

use super::palaces::Palace;
use super::Cuc;

/// Giới tính của người được xem. Về chiều "âm dương thuận nghịch" trong các quy
/// tắc an sao (Hỏa-Linh, Trường Sinh, Đại/Tiểu hạn), Nam = +1, Nữ = -1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Gender {
    Nam,
    Nu,
}

impl Gender {
    /// +1 cho Nam, -1 cho Nữ — multiplicative sign theo quy ước `lasotuvi`.
    pub fn sign_i64(self) -> i64 {
        match self {
            Self::Nam => 1,
            Self::Nu => -1,
        }
    }

    /// Tên tiếng Việt hiển thị.
    pub fn name_vn(self) -> &'static str {
        match self {
            Self::Nam => "Nam",
            Self::Nu => "Nữ",
        }
    }
}

impl std::fmt::Display for Gender {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name_vn())
    }
}

/// Lá số Tử Vi Đẩu Số đầy đủ sau khi [`crate::tuvi::lap_la_so`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TuViChart {
    /// Ngày giờ sinh Dương lịch gốc.
    pub birth: BirthMoment,
    /// Giới tính.
    pub gender: Gender,
    /// Ngày giờ sinh Âm lịch (tính từ `birth.solar_date` theo UTC+7).
    pub lunar: LunarDate,
    /// Cục số đã tính cho lá số.
    pub cuc: Cuc,
    /// 12 cung theo thứ tự Mệnh → Phụ Mẫu → Phúc Đức → Điền Trạch → Quan
    /// Lộc → Nô Bộc → Thiên Di → Tật Ách → Tài Bạch → Tử Tức → Phu Thê →
    /// Huynh Đệ. `palaces[0]` luôn là cung Mệnh.
    pub palaces: [Palace; 12],
    /// Địa Chi của cung Mệnh — tiện cho downstream.
    pub menh_branch: EarthlyBranch,
    /// Địa Chi của cung Thân.
    pub than_branch: EarthlyBranch,
}

/// Lỗi của module Tử Vi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuViError {
    /// Bọc lỗi từ lõi âm lịch (out-of-range, invalid).
    Lunar(LunarError),
    /// Năm sinh ngoài phạm vi 1900–2100.
    OutOfRange(String),
    /// Bất kỳ lỗi nội bộ nào khác (sẽ không gặp nếu input hợp lệ).
    Internal(String),
}

impl std::fmt::Display for TuViError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lunar(e) => write!(f, "lỗi âm lịch: {e}"),
            Self::OutOfRange(s) => write!(f, "ngoài phạm vi: {s}"),
            Self::Internal(s) => write!(f, "lỗi nội bộ: {s}"),
        }
    }
}

impl std::error::Error for TuViError {}