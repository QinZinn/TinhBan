//! Các vòng Trường Sinh (12 sao) cho lá số: Trường Sinh, Mộc Dục, Quan Đới,
//! Lâm Quan, Đế Vượng, Suy, Bệnh, Tử, Mộ, Tuyệt, Thai, Dưỡng.
//!
//! Quy tắc an (theo `lasotuvi.App::lapDiaBan`, "Đã sửa" theo cụ Thiên Lương):
//! - Vị trí khởi **Trường Sinh** phụ thuộc vào Cục:
//!   - Hỏa lục Cục (so=6) → Dần (index 2)
//!   - Kim tứ Cục (so=4) → Tỵ (index 5)
//!   - Thủy nhị Cục (so=2) hoặc Thổ ngũ Cục (so=5) → Thân (index 8)
//!   - Mộc tam Cục (so=3) → Hợi (index 11)
//! - Chiều: **Dương nam / Âm nữ thuận** (tiến theo index), **Âm nam / Dương
//!   nữ nghịch** (lùi theo index). 12 state không trùng: Trường Sinh → Mộc
//!   Dục → Quan Đới → Lâm Quan → Đế Vượng → Suy → Bệnh → Tử → Mộ → Tuyệt →
//!   Thai → Dưỡng → (quay về Trường Sinh).

use crate::EarthlyBranch;

/// 12 state của vòng Trường Sinh, theo thứ tự chuẩn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TruongSinhState {
    TruongSinh = 0,
    MocDuc = 1,
    QuanDoi = 2,
    LamQuan = 3,
    DeVuong = 4,
    Suy = 5,
    Benh = 6,
    Tu = 7,
    Mo = 8,
    Tuyet = 9,
    Thai = 10,
    Duong = 11,
}

impl TruongSinhState {
    /// Index 0..11 trong vòng (theo thứ tự chuẩn).
    pub fn index(self) -> u8 {
        self as u8
    }

    /// Tên tiếng Việt display.
    pub fn name_vn(self) -> &'static str {
        match self {
            Self::TruongSinh => "Trường Sinh",
            Self::MocDuc => "Mộc Dục",
            Self::QuanDoi => "Quan Đới",
            Self::LamQuan => "Lâm Quan",
            Self::DeVuong => "Đế Vượng",
            Self::Suy => "Suy",
            Self::Benh => "Bệnh",
            Self::Tu => "Tử",
            Self::Mo => "Mộ",
            Self::Tuyet => "Tuyệt",
            Self::Thai => "Thai",
            Self::Duong => "Dưỡng",
        }
    }
}

impl std::fmt::Display for TruongSinhState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name_vn())
    }
}

/// Vị trí khởi Trường Sinh theo Cục:
/// - Hỏa lục (6) → Dần
/// - Kim tứ (4) → Tỵ
/// - Thủy nhị (2), Thổ ngũ (5) → Thân
/// - Mộc tam (3) → Hợi
pub fn start_branch_of(cuc_so: u8) -> EarthlyBranch {
    match cuc_so {
        6 => EarthlyBranch::Dan,
        4 => EarthlyBranch::Ty2,
        2 | 5 => EarthlyBranch::Than,
        3 => EarthlyBranch::Hoi,
        _ => panic!("cuc_so out of range 2..=6: {cuc_so}"),
    }
}

/// Trả về `[(TruongSinhState, EarthlyBranch); 12]` — 12 state kèm vị trí Chi,
/// theo chiều `reverse` (true = nghịch, false = thuận).
pub fn truong_sinh_positions(
    cuc_so: u8,
    _menh_branch: EarthlyBranch,
    reverse: bool,
) -> [(TruongSinhState, EarthlyBranch); 12] {
    let start = start_branch_of(cuc_so);
    let states = [
        TruongSinhState::TruongSinh,
        TruongSinhState::MocDuc,
        TruongSinhState::QuanDoi,
        TruongSinhState::LamQuan,
        TruongSinhState::DeVuong,
        TruongSinhState::Suy,
        TruongSinhState::Benh,
        TruongSinhState::Tu,
        TruongSinhState::Mo,
        TruongSinhState::Tuyet,
        TruongSinhState::Thai,
        TruongSinhState::Duong,
    ];

    let mut out = [(TruongSinhState::TruongSinh, EarthlyBranch::Ty); 12];
    let start_idx = start.index() as i64;
    for (i, &state) in states.iter().enumerate() {
        let offset = if reverse {
            -(i as i64)
        } else {
            i as i64
        };
        let branch_idx = (start_idx + offset).rem_euclid(12) as u8;
        out[i] = (state, EarthlyBranch::from_index(branch_idx).unwrap());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_branch_for_each_cuc() {
        assert_eq!(start_branch_of(6), EarthlyBranch::Dan);
        assert_eq!(start_branch_of(4), EarthlyBranch::Ty2);
        assert_eq!(start_branch_of(2), EarthlyBranch::Than);
        assert_eq!(start_branch_of(5), EarthlyBranch::Than);
        assert_eq!(start_branch_of(3), EarthlyBranch::Hoi);
    }

    #[test]
    fn thuan_visits_all_12_branches_starting_from_start() {
        let pos = truong_sinh_positions(2, EarthlyBranch::Ty, false);
        assert_eq!(pos[0].1, EarthlyBranch::Than);
        assert_eq!(pos[1].1, EarthlyBranch::Dau);
        assert_eq!(pos[11].1, EarthlyBranch::Mui);
        // 12 vị trí khác nhau
        let mut branches: Vec<_> = pos.iter().map(|(_, b)| *b).collect();
        branches.sort();
        assert_eq!(branches.len(), 12);
    }

    #[test]
    fn nghich_visits_all_12_branches_in_reverse() {
        let pos = truong_sinh_positions(3, EarthlyBranch::Ty, true);
        assert_eq!(pos[0].1, EarthlyBranch::Hoi);
        assert_eq!(pos[1].1, EarthlyBranch::Tuat);
        assert_eq!(pos[11].1, EarthlyBranch::Ty);
    }
}