//! Thập Thần (Ten Gods) — quan hệ giữa một Can và Nhật Chủ (Can ngày) theo
//! ngũ hành sinh/khắc + âm/dương.
//!
//! # Quy tắc
//!
//! Cho `nhật_chủ` (Can của trụ ngày) và `can` (Can khác bất kỳ), tính `TenGod`:
//!
//! | Quan hệ ngũ hành       | Cùng âm dương | Khác âm dương |
//! |------------------------|---------------|---------------|
//! | Cùng hành              | Tỷ Kiên       | Kiếp Tài      |
//! | Nhật chủ sinh can      | Thực Thần     | Thương Quan   |
//! | Nhật chủ khắc can      | Thiên Tài     | Chính Tài     |
//! | Can khắc Nhật chủ      | Thất Sát       | Chính Quan    |
//! | Can sinh Nhật chủ      | Thiên Ấn       | Chính Ấn      |
//!
//! (Đối chiếu trực tiếp với spec giai đoạn 4.)
//!
//! # Ngũ hành Sinh/Khắc cycle
//!
//! Sinh (生): Mộc→Hỏa→Thổ→Kim→Thủy→Mộc
//! Khắc (剋): Mộc→Thổ→Thủy→Hỏa→Kim→Mộc

use crate::{HeavenlyStem, NguHanh};

/// 10 Thập Thần. Tên tiếng Việt theo chuẩn hoá trong spec giai đoạn 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TenGod {
    TyKien,       // Tỷ Kiên
    KiepTai,      // Kiếp Tài
    ThucThan,     // Thực Thần
    ThuongQuan,   // Thương Quan
    ThienTai,     // Thiên Tài
    ChinhTai,     // Chính Tài
    ThatSat,      // Thất Sát (= Thiên Quan)
    ChinhQuan,   // Chính Quan
    ThienAn,      // Thiên Ấn (= Kiêu Thần)
    ChinhAn,      // Chính Ấn
}

impl TenGod {
    /// Tên tiếng Việt display.
    pub fn name_vn(self) -> &'static str {
        match self {
            Self::TyKien => "Tỷ Kiên",
            Self::KiepTai => "Kiếp Tài",
            Self::ThucThan => "Thực Thần",
            Self::ThuongQuan => "Thương Quan",
            Self::ThienTai => "Thiên Tài",
            Self::ChinhTai => "Chính Tài",
            Self::ThatSat => "Thất Sát",
            Self::ChinhQuan => "Chính Quan",
            Self::ThienAn => "Thiên Ấn",
            Self::ChinhAn => "Chính Ấn",
        }
    }
}

impl std::fmt::Display for TenGod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name_vn())
    }
}

/// Trả `true` nếu Can là Dương (Giáp, Bính, Mậu, Canh, Nhâm); `false` nếu Âm.
fn is_yang_stem(stem: HeavenlyStem) -> bool {
    use HeavenlyStem::*;
    matches!(stem, Giap | Binh | Mau | Canh | Nham)
}

/// `a_hanh sinh b_hanh` (e.g., Mộc→Hỏa, Hỏa→Thổ, Thổ→Kim, Kim→Thủy, Thủy→Mộc).
fn sinh(a: NguHanh, b: NguHanh) -> bool {
    use NguHanh::*;
    matches!(
        (a, b),
        (Moc, Hoa) | (Hoa, Tho) | (Tho, Kim) | (Kim, Thuy) | (Thuy, Moc)
    )
}

/// `a_hanh khắc b_hanh` (e.g., Mộc khắc Thổ, Thổ khắc Thủy, v.v.).
fn khac(a: NguHanh, b: NguHanh) -> bool {
    use NguHanh::*;
    matches!(
        (a, b),
        (Moc, Tho) | (Tho, Thuy) | (Thuy, Hoa) | (Hoa, Kim) | (Kim, Moc)
    )
}

/// Tính Thập Thần của `can` so với `nhật_chủ` (Can của trụ Ngày).
pub fn ten_god_of(nhat_chu: HeavenlyStem, can: HeavenlyStem) -> TenGod {
    let nhat_hanh = crate::nguhanh_of_stem(nhat_chu);
    let can_hanh = crate::nguhanh_of_stem(can);
    let same_polarity = is_yang_stem(nhat_chu) == is_yang_stem(can);

    if nhat_hanh == can_hanh {
        if same_polarity {
            TenGod::TyKien
        } else {
            TenGod::KiepTai
        }
    } else if sinh(nhat_hanh, can_hanh) {
        // Nhật chủ sinh ra can (sinh xuất / "ta sinh")
        if same_polarity {
            TenGod::ThucThan
        } else {
            TenGod::ThuongQuan
        }
    } else if sinh(can_hanh, nhat_hanh) {
        // Can sinh ra Nhật chủ (sinh nhập / "sinh ta")
        if same_polarity {
            TenGod::ThienAn
        } else {
            TenGod::ChinhAn
        }
    } else if khac(nhat_hanh, can_hanh) {
        // Nhật chủ khắc can (khắc xuất / "ta khắc")
        if same_polarity {
            TenGod::ThienTai
        } else {
            TenGod::ChinhTai
        }
    } else if khac(can_hanh, nhat_hanh) {
        // Can khắc Nhật chủ (khắc nhập / "khắc ta")
        if same_polarity {
            TenGod::ThatSat
        } else {
            TenGod::ChinhQuan
        }
    } else {
        // 5 hành đã có tất cả combinations Sinh/Khắc/Cùng-hành → unreachable
        unreachable!("5 han (Kim/Moc/Thuy/Hoa/Tho) has only same/sinh/khac relations \
                      (10 combinations exhausted); can={:?} nhat={:?}", can, nhat_chu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn giap_vs_other_stems_table() {
        // Nhật chủ = Giáp (Dương Mộc)
        use HeavenlyStem::*;
        let expected = [
            (Giap, TenGod::TyKien),   // cùng hành, cùng dương
            (At, TenGod::KiepTai),    // cùng hành, khác âm-dương
            (Binh, TenGod::ThucThan), // Mộc→Hỏa, cùng dương
            (Dinh, TenGod::ThuongQuan), // Mộc→Hỏa, khác
            (Mau, TenGod::ThienTai),  // Mộc khắc Thổ, cùng dương
            (Ky, TenGod::ChinhTai),   // Mộc khắc Thổ, khác
            (Canh, TenGod::ThatSat),  // Kim khắc Mộc, cùng dương
            (Tan, TenGod::ChinhQuan), // Kim khắc Mộc, khác
            (Nham, TenGod::ThienAn),  // Thủy sinh Mộc, cùng dương
            (Quy, TenGod::ChinhAn),   // Thủy sinh Mộc, khác
        ];
        for (stem, exp) in expected {
            let got = ten_god_of(Giap, stem);
            assert_eq!(got, exp, "Nhật chủ Giáp vs {:?}: got {:?}, expected {:?}", stem, got, exp);
        }
    }

#[test]
    fn dinh_nhat_chu_table() {
        // Nhật chủ = Đinh (Âm Hỏa).
        use HeavenlyStem::*;
        let expected = [
            // "Hành đó sinh Nhật chủ" (Mộc→Hỏa) → ['. Ấn', 'Ấn']
            (At, TenGod::ThienAn),    // Mộc→Hỏa, Ấtâm Đinhâm → cùng → Thiên Ấn
            (Giap, TenGod::ChinhAn),   // Mộc→Hỏa, Giáp dương Đinh âm → khác → Chính Ấn
            // Cùng hành Hỏa:
            (Binh, TenGod::KiepTai),   // cùng hành, Bính dương Đinh âm → khác → Kiếp Tài
            (Dinh, TenGod::TyKien),    // cùng hành, cùng âm → Tỷ Kiên
            // "Nhật chủ sinh ra hành đó" (Hỏa→Thổ) → ['Thực', 'Thương']
            (Mau, TenGod::ThuongQuan), // Hỏa→Thổ, Mậu dương Đinh âm → khác → Thương Quan
            (Ky, TenGod::ThucThan),    // Hỏa→Thổ, Kỷ âm Đinh âm → cùng → Thực Thần
            // "Nhật chủ khắc hành đó" (Hỏa→Kim) → ['Thiên Tài', 'Chính Tài']
            (Canh, TenGod::ChinhTai),  // Hỏa→Kim, Canh dương Đinh âm → khác → Chính Tài
            (Tan, TenGod::ThienTai),   // Hỏa→Kim, Tân âm Đinh âm → cùng → Thiên Tài
            // "Hành đó khắc Nhật chủ" (Thủy→Hỏa) → ['Chính Quan', 'Thất Sát']
            (Nham, TenGod::ChinhQuan), // Thủy→Hỏa, Nhâm dương Đinh âm → khác → Chính Quan
            (Quy, TenGod::ThatSat),    // Thủy→Hỏa, Quý âm Đinh âm → cùng → Thất Sát
        ];
for (stem, exp) in expected {
        let got = ten_god_of(Dinh, stem);
        assert_eq!(
            got, exp,
            "Nhật chủ Đinh vs {:?}: got {:?}, expected {:?}", stem, got, exp
        );
    }
}

    #[test]
    fn sinh_cycle_is_correct() {
        use NguHanh::*;
        assert!(sinh(Moc, Hoa));
        assert!(sinh(Hoa, Tho));
        assert!(sinh(Tho, Kim));
        assert!(sinh(Kim, Thuy));
        assert!(sinh(Thuy, Moc));
        assert!(!sinh(Moc, Kim));
        assert!(!sinh(Hoa, Moc));
    }

    #[test]
    fn khac_cycle_is_correct() {
        use NguHanh::*;
        assert!(khac(Moc, Tho));
        assert!(khac(Tho, Thuy));
        assert!(khac(Thuy, Hoa));
        assert!(khac(Hoa, Kim));
        assert!(khac(Kim, Moc));
        assert!(!khac(Moc, Hoa));
        assert!(!khac(Kim, Thuy));
    }

    #[test]
    fn ten_god_enum_names_are_vietnamese() {
        assert_eq!(TenGod::TyKien.name_vn(), "Tỷ Kiên");
        assert_eq!(TenGod::ThatSat.name_vn(), "Thất Sát");
    }
}