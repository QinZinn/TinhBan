//! Cấu trúc dữ liệu công khai của module Bát Tự.

use crate::{BirthMoment, CanChi, EarthlyBranch, HeavenlyStem, LunarError};

use super::thap_than::TenGod;

/// Giới tính (re-export cùng enum với `tuvi::Gender` để dùng chung).
pub use crate::tuvi::Gender;

/// Một trụ (Pillar) trong lá số Bát Tự: Can + Chi + Thập Thần + Tàng Can.
#[derive(Debug, Clone)]
pub struct Pillar {
    /// Cặp Can-Chi của trụ (4 trụ: năm, tháng, ngày, giờ).
    pub can_chi: CanChi,
    /// Thập Thần của **Thiên Can** trụ này so với Nhật Chủ (Can của trụ Ngày).
    /// `None` cho trụ Ngày (đó chính là Nhật Chủ — không có "thập" so với chính
    /// nó).
    pub ten_god: Option<TenGod>,
    /// Tàng Can (Hidden Stems) của Địa Chi trụ này, kèm Thập Thần tương ứng của
    /// từng Can tàng so với Nhật Chủ. Rỗng (`vec![]`) không xảy ra nếu branch
    /// hợp lệ (mọi Chi đều có ≥1 Tàng Can).
    pub hidden_stems: Vec<(HeavenlyStem, TenGod)>,
}

impl Pillar {
    /// Shortcut: Thiên Can của trụ.
    pub fn stem(&self) -> HeavenlyStem {
        self.can_chi.stem
    }
    /// Shortcut: Địa Chi của trụ.
    pub fn branch(&self) -> EarthlyBranch {
        self.can_chi.branch
    }
}

/// Lá số Bát Tự đầy đủ (Tứ Trụ + thống kê + Thập Thần).
#[derive(Debug, Clone)]
pub struct BatTuChart {
    /// Ngày giờ sinh Dương lịch gốc.
    pub birth: BirthMoment,
    /// Giới tính (lưu để dành cho Đại Vận / mở rộng sau).
    pub gender: Gender,
    /// Trụ Năm (Bát Tự year — có thể lệch 1 năm so với Dương lịch nếu sinh
    /// trước Lập Xuân).
    pub year_pillar: Pillar,
    /// Trụ Tháng (theo tháng Bát Tự = tháng tiết khí, không phải tháng âm
    /// lịch Tử Vi).
    pub month_pillar: Pillar,
    /// Trụ Ngày (= Nhật Chủ — Thập Thần của trụ này là `None`).
    pub day_pillar: Pillar,
    /// Trụ Giờ.
    pub hour_pillar: Pillar,
    /// Thống kê Ngũ Hành trong 8 chữ (4 Can + 4 Chi, mỗi Can/Chi đóng góp Hành
    /// chính của nó). Chiếc single count = 8.
    pub nguhanh_count: NguHanhCount,
}

/// Số lần xuất hiện mỗi hành trong 8 chữ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NguHanhCount {
    pub kim: u8,
    pub moc: u8,
    pub thuy: u8,
    pub hoa: u8,
    pub tho: u8,
}

impl NguHanhCount {
    /// Tổng số = 8 (4 Can + 4 Chi). Mỗi Can-Chi counted 1 lần theo hành chính.
    pub fn total(&self) -> u8 {
        self.kim + self.moc + self.thuy + self.hoa + self.tho
    }
}

/// Lỗi của module Bát Tự.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatTuError {
    /// Bọc lỗi từ lõi âm lịch (out-of-range, invalid).
    Lunar(LunarError),
    /// Năm sinh ngoài phạm vi 1900–2100.
    OutOfRange(String),
    /// Lỗi nội bộ (không gặp nếu input hợp lệ).
    Internal(String),
}

impl std::fmt::Display for BatTuError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lunar(e) => write!(f, "lỗi âm lịch: {e}"),
            Self::OutOfRange(s) => write!(f, "ngoài phạm vi: {s}"),
            Self::Internal(s) => write!(f, "lỗi nội bộ: {s}"),
        }
    }
}

impl std::error::Error for BatTuError {}

impl From<LunarError> for BatTuError {
    fn from(e: LunarError) -> Self {
        Self::Lunar(e)
    }
}