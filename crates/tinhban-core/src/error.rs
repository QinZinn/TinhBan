//! Lỗi cho lõi âm lịch & Can Chi.

use std::fmt;

/// Lỗi phát sinh trong các hàm chuyển / tính Can Chi của `tinhban-core`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LunarError {
    /// Ngày / năm ngoài phạm vi 1900–2100 (Dương lịch).
    OutOfRange(String),
    /// Ngày tháng âm lịch không hợp lệ (ngày > 30, tháng > 12, hoặc tháng
    /// nhuận không tồn tại ở năm đó).
    InvalidLunarDate(String),
}

impl fmt::Display for LunarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange(msg) => write!(f, "ngoài phạm vi: {msg}"),
            Self::InvalidLunarDate(msg) => write!(f, "ngày âm lịch không hợp lệ: {msg}"),
        }
    }
}

impl std::error::Error for LunarError {}