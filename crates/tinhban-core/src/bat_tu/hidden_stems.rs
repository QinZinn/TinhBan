//! Bảng Địa Chi Tàng Can (Hidden Stems) — bảng tra cố định cho 12 Địa Chi.
//!
//! Mỗi Chi có 1-3 Can ẩn (Tàng Can). Quy ước đặt theo chuẩn Bát Tự Bắc Tông:
//!   - **Bản khí** (= Can chính, khí chủ đạo của Chi, listed đầu tiên)
//!   - **Trung khí** (= Can phụthứ 2 nếu có)
//!   - **Dư khí** (= Can phụthứ 3 nếu có)
//!
//! Bảng đối chiếu [`lunar-rs`](https://crates.io/crates/lunar-rs) v1.0.0-rc1,
//! và các reference chuẩn trên nhiều source Bát Tự cộng đồng.
//!
//! | Chi    | Tàng Can (bản → trung → dư)          |
//! |--------|--------------------------------------|
//! | Tý     | Quý                                  |
//! | Sửu    | Kỷ, Quý, Tân                         |
//! | Dần    | Giáp, Bính, Mậu                       |
//! | Mão    | Ất                                   |
//! | Thìn   | Mậu, Ất, Quý                          |
//! | Tỵ     | Bính, Mậu, Canh                       |
//! | Ngọ    | Đinh, Kỷ                             |
//! | Mùi    | Kỷ, Đinh, Ất                          |
//! | Thân   | Canh, Nhâm, Mậu                       |
//! | Dậu    | Tân                                  |
//! | Tuất   | Mậu, Tân, Đinh                        |
//! | Hợi    | Nhâm, Giáp                           |

use crate::{EarthlyBranch, HeavenlyStem};

/// Trả về danh sách Tàng Can của `branch`, theo thứ tự [bản khí, trung khí, dư
/// khí]. Đa số Chi có 1-3 Can tàng; Chi Mão, Tý, Dậu chỉ có 1 Can tàng.
///
/// Trường hợp branch::from_index thành công (luôn đúng cho input chuẩn).
pub fn hidden_stems(branch: EarthlyBranch) -> &'static [HeavenlyStem] {
    const TABLE: [&[HeavenlyStem]; 12] = [
        &[HeavenlyStem::Quy],                 // Tý
        &[HeavenlyStem::Ky, HeavenlyStem::Quy, HeavenlyStem::Tan],        // Sửu
        &[HeavenlyStem::Giap, HeavenlyStem::Binh, HeavenlyStem::Mau],     // Dần
        &[HeavenlyStem::At],                  // Mão
        &[HeavenlyStem::Mau, HeavenlyStem::At, HeavenlyStem::Quy],        // Thìn
        &[HeavenlyStem::Binh, HeavenlyStem::Mau, HeavenlyStem::Canh],     // Tỵ
        &[HeavenlyStem::Dinh, HeavenlyStem::Ky],            // Ngọ
        &[HeavenlyStem::Ky, HeavenlyStem::Dinh, HeavenlyStem::At],        // Mùi
        &[HeavenlyStem::Canh, HeavenlyStem::Nham, HeavenlyStem::Mau],     // Thân
        &[HeavenlyStem::Tan],                 // Dậu
        &[HeavenlyStem::Mau, HeavenlyStem::Tan, HeavenlyStem::Dinh],      // Tuất
        &[HeavenlyStem::Nham, HeavenlyStem::Giap],          // Hợi
    ];
    TABLE[branch.index() as usize]
}

/// Trả về (bản khí, trung khí, dư khí) dưới dạng tuple `(bản, Option<trung>,
/// Option<dư>)` — tiện cho callers muốn truy cập theo vị trí thay vì slice.
pub fn hidden_stems_tuple(branch: EarthlyBranch) -> (HeavenlyStem, Option<HeavenlyStem>, Option<HeavenlyStem>) {
    let stems = hidden_stems(branch);
    let ban = stems[0];
    let trung = stems.get(1).copied();
    let du = stems.get(2).copied();
    (ban, trung, du)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EarthlyBranch;

    fn all_12_branches() -> [EarthlyBranch; 12] {
        use EarthlyBranch::*;
        [
            Ty, Suu, Dan, Mao, Thin, Ty2, Ngo, Mui, Than, Dau, Tuat, Hoi,
        ]
    }

    #[test]
    fn all_12_branches_have_at_least_one_hidden_stem() {
        for b in all_12_branches() {
            let stems = hidden_stems(b);
            assert!(!stems.is_empty(), "branch {:?} has 0 hidden stems", b);
            assert!(stems.len() <= 3, "branch {:?} has > 3 stems", b);
        }
    }

    #[test]
    fn table_matches_standard_reference() {
        use crate::HeavenlyStem::*;
        use EarthlyBranch::*;
        let cases: [(EarthlyBranch, &[crate::HeavenlyStem]); 12] = [
            (Ty, &[Quy]),
            (Suu, &[Ky, Quy, Tan]),
            (Dan, &[Giap, Binh, Mau]),
            (Mao, &[At]),
            (Thin, &[Mau, At, Quy]),
            (Ty2, &[Binh, Mau, Canh]),
            (Ngo, &[Dinh, Ky]),
            (Mui, &[Ky, Dinh, At]),
            (Than, &[Canh, Nham, Mau]),
            (Dau, &[Tan]),
            (Tuat, &[Mau, Tan, Dinh]),
            (Hoi, &[Nham, Giap]),
        ];
        for (branch, expected) in cases {
            let got = hidden_stems(branch);
            assert_eq!(got.len(), expected.len(), "hidden stems of {:?} mismatch", branch);
            for (i, &s) in expected.iter().enumerate() {
                assert_eq!(got[i], s, "hidden stems of {:?} at {}: mismatch", branch, i);
            }
        }
    }
}