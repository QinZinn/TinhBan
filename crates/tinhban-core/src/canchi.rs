//! Can–Chi (Thiên Can × Địa Chi), Ngũ Hành, và các quy tắc "Ngũ Thử Độn" cho
//! tháng và giờ. Toán đối chiếu `J2TEAM/vibe.j2team.org/.../lunar.ts`,
//! `quocthang0507/Calendar/CalendarLib/Lunar.cs` và nhiều port khác.
//!
//! # Quy ước index
//!
//! - 10 Can: `HeavenlyStem` index 0..9 = Giáp, Ất, Bính, Đinh, Mậu, Kỷ, Canh,
//!   Tân, Nhâm, Quý.
//! - 12 Chi: `EarthlyBranch` index 0..11 = Tý, Sửu, Dần, Mão, Thìn, Tỵ, Ngọ,
//!   Mùi, Thân, Dậu, Tuất, Hợi.
//! - 5 Hành: `NguHanh` (Kim/Mộc/Thủy/Hỏa/Thổ), thứ tự tra trong bảng dưới.

use crate::CanChi;

// ===========================================================================
// Enums
// ===========================================================================

/// 10 Thiên Can (Heavenly Stems). Thứ tự cố định, index 0..9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HeavenlyStem {
    Giap = 0,
    At = 1,
    Binh = 2,
    Dinh = 3,
    Mau = 4,
    Ky = 5,
    Canh = 6,
    Tan = 7,
    Nham = 8,
    Quy = 9,
}

impl HeavenlyStem {
    /// Index 0..9 (= giá trị `repr(u8)`).
    pub fn index(self) -> u8 {
        self as u8
    }

    /// Tên tiếng Việt đầy đủ có dấu (display).
    pub fn name_vn(self) -> &'static str {
        match self {
            Self::Giap => "Giáp",
            Self::At => "Ất",
            Self::Binh => "Bính",
            Self::Dinh => "Đinh",
            Self::Mau => "Mậu",
            Self::Ky => "Kỷ",
            Self::Canh => "Canh",
            Self::Tan => "Tân",
            Self::Nham => "Nhâm",
            Self::Quy => "Quý",
        }
    }

    /// Chiều đảo, trả Can theo index 0..9. Trả `None` nếu ngoài phạm vi.
    pub fn from_index(i: u8) -> Option<Self> {
        Some(match i {
            0 => Self::Giap,
            1 => Self::At,
            2 => Self::Binh,
            3 => Self::Dinh,
            4 => Self::Mau,
            5 => Self::Ky,
            6 => Self::Canh,
            7 => Self::Tan,
            8 => Self::Nham,
            9 => Self::Quy,
            _ => return None,
        })
    }
}

/// 12 Địa Chi (Earthly Branches). Thứ tự cố định, index 0..11.
///
/// Đánh tên `Ty2` cho chi "Tỵ" (snake, miền Nam Việt Nam hay đọc "Tí" / Tỵ)
/// để tránh trùng `Ty` (Tý = rat, index 0). Quy ước này theo gợi ý của spec
/// giai đoạn 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EarthlyBranch {
    Ty = 0,
    Suu = 1,
    Dan = 2,
    Mao = 3,
    Thin = 4,
    Ty2 = 5,
    Ngo = 6,
    Mui = 7,
    Than = 8,
    Dau = 9,
    Tuat = 10,
    Hoi = 11,
}

impl EarthlyBranch {
    /// Index 0..11 (= giá trị `repr(u8)`).
    pub fn index(self) -> u8 {
        self as u8
    }

    /// Tên tiếng Việt đầy đủ có dấu (display).
    pub fn name_vn(self) -> &'static str {
        match self {
            Self::Ty => "Tý",
            Self::Suu => "Sửu",
            Self::Dan => "Dần",
            Self::Mao => "Mão",
            Self::Thin => "Thìn",
            Self::Ty2 => "Tỵ",
            Self::Ngo => "Ngọ",
            Self::Mui => "Mùi",
            Self::Than => "Thân",
            Self::Dau => "Dậu",
            Self::Tuat => "Tuất",
            Self::Hoi => "Hợi",
        }
    }

    /// Chiều đảo, trả Branch theo index 0..11. Trả `None` nếu ngoài phạm vi.
    pub fn from_index(i: u8) -> Option<Self> {
        Some(match i {
            0 => Self::Ty,
            1 => Self::Suu,
            2 => Self::Dan,
            3 => Self::Mao,
            4 => Self::Thin,
            5 => Self::Ty2,
            6 => Self::Ngo,
            7 => Self::Mui,
            8 => Self::Than,
            9 => Self::Dau,
            10 => Self::Tuat,
            11 => Self::Hoi,
            _ => return None,
        })
    }
}

/// Ngũ Hành (Five Elements).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NguHanh {
    Kim,
    Moc,
    Thuy,
    Hoa,
    Tho,
}

impl NguHanh {
    /// Tên tiếng Việt có dấu (display).
    pub fn name_vn(self) -> &'static str {
        match self {
            Self::Kim => "Kim",
            Self::Moc => "Mộc",
            Self::Thuy => "Thủy",
            Self::Hoa => "Hỏa",
            Self::Tho => "Thổ",
        }
    }
}

/// Alias topic — synonym `NguHanhElement` để dễ chọn cho downstream crate.
pub type NguHanhElement = NguHanh;

impl std::fmt::Display for NguHanh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name_vn())
    }
}

// ===========================================================================
// Can–Chi quy tắc (year / month / day / hour)
// ===========================================================================

/// Can–Chi của **năm** (lunar year).
/// Công thức: `can = (year + 6) % 10`, `chi = (year + 8) % 12`.
pub fn year_can_chi(year: i64) -> CanChi {
    let can_i = ((year + 6).rem_euclid(10)) as u8;
    let chi_i = ((year + 8).rem_euclid(12)) as u8;
    CanChi {
        stem: HeavenlyStem::from_index(can_i).unwrap(),
        branch: EarthlyBranch::from_index(chi_i).unwrap(),
    }
}

/// Can–Chi của **tháng** âm theo quy tắc "Ngũ Thử Độn":
/// - `yearCan = (lunarYear + 6) % 10`
/// - `baseCan = ((yearCan % 5) * 2 + 2) % 10`
/// - `can = (baseCan + month - 1) % 10`
/// - `chi = (month + 1) % 12`  (tháng 1 → Dần, tháng 2 → Mão, ...)
///
/// Bảng quy tắc truyền thống:
/// - năm Giáp / Kỷ → tháng 1 Bính Dần
/// - năm Ất / Canh → tháng 1 Mậu Dần
/// - năm Bính / Tân → tháng 1 Canh Dần
/// - năm Đinh / Nhâm → tháng 1 Nhâm Dần
/// - năm Mậu / Quý → tháng 1 Giáp Dần
pub fn month_can_chi(lunar_year_can: HeavenlyStem, lunar_month: u8) -> CanChi {
    let year_can_i = lunar_year_can.index() as i64;
    let base = ((year_can_i % 5) * 2 + 2).rem_euclid(10) as u8;
    let can = (base as i64 + lunar_month as i64 - 1).rem_euclid(10) as u8;
    let chi = ((lunar_month as i64 + 1) % 12) as u8;
    CanChi {
        stem: HeavenlyStem::from_index(can).unwrap(),
        branch: EarthlyBranch::from_index(chi).unwrap(),
    }
}

/// Can–Chi của **ngày** từ Julian Day Number nguyên (trưa UTC của ngày Dương).
/// Công thức: `can = (jd + 9) % 10`, `chi = (jd + 1) % 12`.
pub fn day_can_chi(jd: i64) -> CanChi {
    let can = (jd + 9).rem_euclid(10) as u8;
    let chi = (jd + 1).rem_euclid(12) as u8;
    CanChi {
        stem: HeavenlyStem::from_index(can).unwrap(),
        branch: EarthlyBranch::from_index(chi).unwrap(),
    }
}

/// Can–Chi của **giờ** (12 giờ Địa Chi truyền thống, mỗi giờ = 2 tiếng).
///
/// - `hour` 0..=23 (giờ Dương lịch 24h).
/// - `hourChiIndex = floor((hour + 1) / 2) % 12` → 0=Tý (h exc. 23 và 0),
///   1=Sửu (1..2), ..., 11=Hợi (21..22). `hour=23` cũng Tý (giờ Tý bắt đầu 23h).
/// - Can của giờ theo "Ngũ Thử Độn Thời":
///   - `baseCan = ((dayCan % 5) * 2) % 10`
///   - `can = (baseCan + hourChiIndex) % 10`
///
/// Bảng truyền thống (Can của giờ Tý theo Can của ngày):
/// - ngày Giáp / Kỷ → Giáp Tý
/// - ngày Ất / Canh → Bính Tý
/// - ngày Bính / Tân → Mậu Tý
/// - ngày Đinh / Nhâm → Canh Tý
/// - ngày Mậu / Quý → Nhâm Tý
pub fn hour_can_chi(day_can: HeavenlyStem, hour: u8) -> CanChi {
    let day_can_i = day_can.index() as i64;
    let hour_chi_i = ((hour as i64 + 1) / 2).rem_euclid(12) as u8;
    let base = ((day_can_i % 5) * 2).rem_euclid(10) as u8;
    let can = (base as i64 + hour_chi_i as i64).rem_euclid(10) as u8;
    CanChi {
        stem: HeavenlyStem::from_index(can).unwrap(),
        branch: EarthlyBranch::from_index(hour_chi_i).unwrap(),
    }
}

// ===========================================================================
// Ngũ Hành của Can / Chi
// ===========================================================================

/// Ngũ Hành của Thiên Can:
/// - Giáp, Ất → Mộc
/// - Bính, Đinh → Hỏa
/// - Mậu, Kỷ → Thổ
/// - Canh, Tân → Kim
/// - Nhâm, Quý → Thủy
pub fn nguhanh_of_stem(stem: HeavenlyStem) -> NguHanh {
    use NguHanh::*;
    use HeavenlyStem::*;
    match stem {
        Giap | At => Moc,
        Binh | Dinh => Hoa,
        Mau | Ky => Tho,
        Canh | Tan => Kim,
        Nham | Quy => Thuy,
    }
}

/// Ngũ Hành của Địa Chi (theo mùa):
/// - Dần, Mão → Mộc
/// - Tỵ, Ngọ → Hỏa
/// - Thân, Dậu → Kim
/// - Tý, Hợi → Thủy
/// - Sửu, Thìn, Mùi, Tuất → Thổ (4 "tháng cuối mùa")
pub fn nguhanh_of_branch(branch: EarthlyBranch) -> NguHanh {
    use NguHanh::*;
    use EarthlyBranch::*;
    match branch {
        Dan | Mao => Moc,
        Ty2 | Ngo => Hoa,
        Than | Dau => Kim,
        Ty | Hoi => Thuy,
        Suu | Thin | Mui | Tuat => Tho,
    }
}

/// In ra chuỗi tiếng Việt "Can Chi", ví dụ "Giáp Thìn", "Bính Dần".
pub fn can_chi_display(cc: CanChi) -> String {
    format!("{} {}", cc.stem.name_vn(), cc.branch.name_vn())
}

// ===========================================================================
// Helper modulo: tests
// ===========================================================================

#[cfg(test)]
mod spec_table_checks {
    use super::*;

    /// Bảng "Ngũ Thử Độn": tháng 1 → Can của tháng 1 theo Can của năm.
    #[test]
    fn ngu_thu_don_month_table_is_correct() {
        // (year stem, expected month-1 stem)
        let table = [
            (HeavenlyStem::Giap, HeavenlyStem::Binh),
            (HeavenlyStem::Ky, HeavenlyStem::Binh),
            (HeavenlyStem::At, HeavenlyStem::Mau),
            (HeavenlyStem::Canh, HeavenlyStem::Mau),
            (HeavenlyStem::Binh, HeavenlyStem::Canh),
            (HeavenlyStem::Tan, HeavenlyStem::Canh),
            (HeavenlyStem::Dinh, HeavenlyStem::Nham),
            (HeavenlyStem::Nham, HeavenlyStem::Nham),
            (HeavenlyStem::Mau, HeavenlyStem::Giap),
            (HeavenlyStem::Quy, HeavenlyStem::Giap),
        ];
        for (year_stem, m1_stem) in table {
            let cc = month_can_chi(year_stem, 1);
            assert_eq!(
                cc.stem, m1_stem,
                "năm Can {:?} → tháng 1 phải có Can {:?} (nhận {:?})",
                year_stem, m1_stem, cc.stem
            );
            assert_eq!(cc.branch, EarthlyBranch::Dan, "tháng 1 → Dần");
        }
    }

    /// Bảng "Ngũ Thử Độn thời": giờ Tý của ngày có Can → Can của giờ Tý.
    #[test]
    fn ngu_thu_don_hour_table_is_correct() {
        let table = [
            (HeavenlyStem::Giap, HeavenlyStem::Giap),
            (HeavenlyStem::Ky, HeavenlyStem::Giap),
            (HeavenlyStem::At, HeavenlyStem::Binh),
            (HeavenlyStem::Canh, HeavenlyStem::Binh),
            (HeavenlyStem::Binh, HeavenlyStem::Mau),
            (HeavenlyStem::Tan, HeavenlyStem::Mau),
            (HeavenlyStem::Dinh, HeavenlyStem::Canh),
            (HeavenlyStem::Nham, HeavenlyStem::Canh),
            (HeavenlyStem::Mau, HeavenlyStem::Nham),
            (HeavenlyStem::Quy, HeavenlyStem::Nham),
        ];
        for (day_stem, ty_stem) in table {
            let cc = hour_can_chi(day_stem, 23); // 23h → giờ Tý
            assert_eq!(
                cc.stem, ty_stem,
                "ngày Can {:?} → giờ Tý phải có Can {:?}",
                day_stem, ty_stem
            );
            assert_eq!(cc.branch, EarthlyBranch::Ty, "hour=23 → Tý");
            let cc0 = hour_can_chi(day_stem, 0); // 0h vẫn Tý
            assert_eq!(cc0.branch, EarthlyBranch::Ty);
            assert_eq!(cc0.stem, ty_stem);
        }
    }

    /// Ngũ hành của 10 Can.
    #[test]
    fn nguhanh_stem_table() {
        use NguHanh::*;
        let table = [
            (HeavenlyStem::Giap, Moc),
            (HeavenlyStem::At, Moc),
            (HeavenlyStem::Binh, Hoa),
            (HeavenlyStem::Dinh, Hoa),
            (HeavenlyStem::Mau, Tho),
            (HeavenlyStem::Ky, Tho),
            (HeavenlyStem::Canh, Kim),
            (HeavenlyStem::Tan, Kim),
            (HeavenlyStem::Nham, Thuy),
            (HeavenlyStem::Quy, Thuy),
        ];
        for (stem, expected) in table {
            assert_eq!(nguhanh_of_stem(stem), expected);
        }
    }

    /// Ngũ hành của 12 Chi.
    #[test]
    fn nguhanh_branch_table() {
        use EarthlyBranch::*;
        use NguHanh::*;
        let table = [
            (Ty, Thuy),
            (Suu, Tho),
            (Dan, Moc),
            (Mao, Moc),
            (Thin, Tho),
            (Ty2, Hoa),
            (Ngo, Hoa),
            (Mui, Tho),
            (Than, Kim),
            (Dau, Kim),
            (Tuat, Tho),
            (Hoi, Thuy),
        ];
        for (branch, expected) in table {
            assert_eq!(
                nguhanh_of_branch(branch),
                expected,
                "branch {:?} → hành {:?}",
                branch,
                expected
            );
        }
    }

    /// Mỗi tháng âm phải ứng với đúng Địa Chi.
    #[test]
    fn month_branch_sequence() {
        // Tháng 1 → Dần, tháng 2 → Mão, ..., tháng 11 → Tý, tháng 12 → Sửu
        let expected_branches = [
            EarthlyBranch::Dan,
            EarthlyBranch::Mao,
            EarthlyBranch::Thin,
            EarthlyBranch::Ty2,
            EarthlyBranch::Ngo,
            EarthlyBranch::Mui,
            EarthlyBranch::Than,
            EarthlyBranch::Dau,
            EarthlyBranch::Tuat,
            EarthlyBranch::Hoi,
            EarthlyBranch::Ty,
            EarthlyBranch::Suu,
        ];
        for (m, chi) in (1..=12u8).zip(expected_branches.iter().copied()) {
            let cc = month_can_chi(HeavenlyStem::Giap, m);
            assert_eq!(cc.branch, chi, "month {}", m);
        }
    }

    /// Giờ Địa Chi phải ứng với đúng song-tiếng.
    #[test]
    fn hour_branch_sequence() {
        // (hour, chi):
        // 23→Tý, 0→Tý, 1–2→Sửu, 3–4→Dần, 5–6→Mão, 7–8→Thìn,
        // 9–10→Tỵ, 11–12→Ngọ, 13–14→Mùi, 15–16→Thân, 17–18→Dậu,
        // 19–20→Tuất, 21–22→Hợi
        let cases: [(u8, EarthlyBranch); 24] = [
            (0, EarthlyBranch::Ty),
            (1, EarthlyBranch::Suu),
            (2, EarthlyBranch::Suu),
            (3, EarthlyBranch::Dan),
            (4, EarthlyBranch::Dan),
            (5, EarthlyBranch::Mao),
            (6, EarthlyBranch::Mao),
            (7, EarthlyBranch::Thin),
            (8, EarthlyBranch::Thin),
            (9, EarthlyBranch::Ty2),
            (10, EarthlyBranch::Ty2),
            (11, EarthlyBranch::Ngo),
            (12, EarthlyBranch::Ngo),
            (13, EarthlyBranch::Mui),
            (14, EarthlyBranch::Mui),
            (15, EarthlyBranch::Than),
            (16, EarthlyBranch::Than),
            (17, EarthlyBranch::Dau),
            (18, EarthlyBranch::Dau),
            (19, EarthlyBranch::Tuat),
            (20, EarthlyBranch::Tuat),
            (21, EarthlyBranch::Hoi),
            (22, EarthlyBranch::Hoi),
            (23, EarthlyBranch::Ty),
        ];
        for (h, chi) in cases {
            let cc = hour_can_chi(HeavenlyStem::Giap, h);
            assert_eq!(cc.branch, chi, "hour {}", h);
        }
    }
}