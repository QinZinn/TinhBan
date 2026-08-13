//! 12 cung (Thập Nhị Cung) và hàm tiện `dich_cung`.
//!
//! `PalaceName` enum đại diện 12 cung theo thứ tự cố định. Lớp `Palace` lưu
//! vị trí (Địa Chi) + tên cung + các sao đóng tại đây + flag Mệnh/Thân.

use crate::EarthlyBranch;
use super::stars::Sao;
use super::truong_sinh::TruongSinhState;

/// Tên 12 cung theo thứ tự cố định Mệnh → Huynh Đệ (theo lasotuvi
/// `cungChuThapNhiCung`). Index 0 = Mệnh, 11 = Huynh Đệ.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PalaceName {
    Menh = 0,
    PhuMau = 1,
    PhucDuc = 2,
    DienTrach = 3,
    QuanLoc = 4,
    NoBoc = 5,
    ThienDi = 6,
    TatAch = 7,
    TaiBach = 8,
    TuTuc = 9,
    PhuThe = 10,
    HuynhDe = 11,
}

impl PalaceName {
    /// 12 cung theo thứ tự cố định (Mệnh, Phụ Mẫu, ..., Huynh Đệ).
    pub fn all_in_order() -> [Self; 12] {
        [
            Self::Menh,
            Self::PhuMau,
            Self::PhucDuc,
            Self::DienTrach,
            Self::QuanLoc,
            Self::NoBoc,
            Self::ThienDi,
            Self::TatAch,
            Self::TaiBach,
            Self::TuTuc,
            Self::PhuThe,
            Self::HuynhDe,
        ]
    }

    /// Tên tiếng Việt display, có dấu.
    pub fn name_vn(self) -> &'static str {
        match self {
            Self::Menh => "Mệnh",
            Self::PhuMau => "Phụ Mẫu",
            Self::PhucDuc => "Phúc Đức",
            Self::DienTrach => "Điền Trạch",
            Self::QuanLoc => "Quan Lộc",
            Self::NoBoc => "Nô Bộc",
            Self::ThienDi => "Thiên Di",
            Self::TatAch => "Tật Ách",
            Self::TaiBach => "Tài Bạch",
            Self::TuTuc => "Tử Tức",
            Self::PhuThe => "Phu Thê",
            Self::HuynhDe => "Huynh Đệ",
        }
    }

    /// Index 0..11.
    pub fn index(self) -> u8 {
        self as u8
    }
}

impl std::fmt::Display for PalaceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name_vn())
    }
}

/// Một cung trên bàn Tử Vi.
#[derive(Debug, Clone)]
pub struct Palace {
    /// Tên cung (Mệnh, Phụ Mẫu, ...).
    pub name: PalaceName,
    /// Địa Chi cố định của cung trên Địa bàn (`EarthlyBranch::index()` 0..11).
    pub branch: EarthlyBranch,
    /// Các sao đóng tại cung (cả chính tinh + phụ tinh).
    pub stars: Vec<Sao>,
    /// Vòng Trường Sinh (các state đóng tại cung — thường 0 hoặc 1).
    pub truong_sinh: Vec<TruongSinhState>,
    /// `true` nếu đây là cung Mệnh.
    pub is_menh: bool,
    /// `true` nếu đây là cung Thân.
    pub is_than: bool,
}

impl Palace {
    pub fn new(name: PalaceName, branch: EarthlyBranch) -> Self {
        Self {
            name,
            branch,
            stars: Vec::new(),
            truong_sinh: Vec::new(),
            is_menh: false,
            is_than: false,
        }
    }

    /// Push sao vào cung. Tránh dup (cùng `Sao` insert vào 1 list nhiều lần
    /// — không tự động, ở phase 3).
    pub fn add_star(&mut self, sao: Sao) {
        self.stars.push(sao);
    }

    /// Trả `true` nếu có sao `sao` ở cung này.
    pub fn has_star(&self, sao: Sao) -> bool {
        self.stars.contains(&sao)
    }
}

/// `dichCung` theo quy ước `lasotuvi` (modulo 12). Dùng 0-based index của
/// `EarthlyBranch`. Trả `u8` trong 0..12.
pub fn dich_cung(base: EarthlyBranch, offset: i64) -> EarthlyBranch {
    let idx = ((base.index() as i64 + offset).rem_euclid(12)) as u8;
    EarthlyBranch::from_index(idx).unwrap()
}