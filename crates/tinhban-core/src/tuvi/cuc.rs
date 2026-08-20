//! Tính Cục (Kim/Mộc/Thủy/Hỏa/Thổ Cục) cho lá số Tử Vi.
//!
//! Cách tính: Lấy Can-Chi của tháng âm lịch "chứa cung Mệnh" → nạp âm → Cục.
//!
//! Algorithm theo [`lasotuvi`](https://github.com/doanguyen/lasotuvi)
//! `AmDuong.py::timCuc`:
//!  1. `canThangGieng = (canNam * 2 + 1) % 10` — Can của tháng Giêng của năm
//!     đó theo quy tắc "Ngũ Hổ Độn" (Bính Dần là tháng 1 của năm Giáp/Kỷ).
//!     *Lưu ý*: lasotuvi dùng `canNam` 1-based (1=Giáp,..10=Quý), `vitriDiaBan`
//!     trong range 1..12. Công thức trên trả về 0..9; HO đổi `if canThangMenh
//!     == 0: canThangMenh = 10` cho hiệu ứng 1-based.
//!  2. `canThangMenh = ((viTriCungMenh - 3) % 12 + canThangGieng) % 10`
//!     Số thành 0..9 (trong lasotuvi là 1..10).
//!  3. `nguHanhNapAm(viTriCungMenh_1to12, canThangMenh_1to10)` — nạp âm cho
//!     cặp Can-Chi → ra một trong K/M/T/H/O.
//!  4. Map K→Kim tứ Cục, M→Mộc tam Cục, T→Thủy nhị Cục, H→Hỏa lục Cục,
//!     O→Thổ ngũ Cục.
//!
//! Port sang Rust: tất cả dùng 0-based (`EarthlyBranch::index()` 0..11,
//! `HeavenlyStem::index()` 0..9), cùng công thức modulo.

use crate::{EarthlyBranch, HeavenlyStem, NguHanh};

/// Cục của lá số + thông tin metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cuc {
    /// Số Cục (2..6) dùng cho vòng Trường Sinh + an Tử Vi.
    pub so: u8,
    /// Hành của Cục.
    pub hanh: NguHanh,
}

impl Cuc {
    /// Tên Cục tiếng Việt display, vd "Thủy nhị Cục".
    pub fn name_vn(self) -> String {
        let so_vn = match self.so {
            2 => "nhị",
            3 => "tam",
            4 => "tứ",
            5 => "ngũ",
            6 => "lục",
            _ => "?",
        };
        format!("{} {} Cục", self.hanh.name_vn(), so_vn)
    }
}

impl std::fmt::Display for Cuc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name_vn())
    }
}

/// Thông tin bổ sung về Cục (mở rộng sau). Hiện chỉ giữ số + hành.
pub type CucInfo = Cuc;

/// Tính Cục từ vị trí cung Mệnh trên Địa bàn + Can của năm sinh.
///
/// `menh_branch`: Địa Chi của cung Mệnh. `year_stem`: Can của năm sinh.
pub fn tinh_cuc(menh_branch: EarthlyBranch, year_stem: HeavenlyStem) -> Cuc {
    // canThangGieng = (canNam * 2 + 1) % 10  — 1-based theo lasotuvi; port 0-based:
    // Trong lasotuvi: canNam 1..10, canThangGieng = (canNam * 2 + 1) % 10
    //   → month-1 (Giáp) của năm Giáp(1): (1*2+1)%10 = 3 → Bính(3) → đúng.
    // Chuyển 0-based (year_stem.index() 0..9):
    //   canThangGieng_0based = ((year_stem + 1) * 2 + 1) % 10
    //                       ... để giữ tương đương (= lasotuvi canNam+1).
    // Đơn giản hơn: tính theo 1-based rồi quy đổi.
    let can_nam_1 = year_stem.index() as i64 + 1; // 1..10
    let can_thang_gieng_1 = (can_nam_1 * 2 + 1).rem_euclid(10); // 0..9 (lasotuvi)
    let can_thang_gieng_1 = if can_thang_gieng_1 == 0 { 10 } else { can_thang_gieng_1 };

    // canThangMenh = ((viTriCungMenh - 3) % 12 + canThangGieng) % 10
    // Trong lasotuvi: viTriCungMenh 1..12, mod 12 với 1-based.
    // Port: menh_branch.index() 0..11, chuyển sang 1-based: +1.
    let vi_tri_menh_1 = menh_branch.index() as i64 + 1;
    let term1 = (vi_tri_menh_1 - 3).rem_euclid(12);
    let mut can_thang_menh_1 = (term1 + can_thang_gieng_1).rem_euclid(10);
    if can_thang_menh_1 == 0 {
        can_thang_menh_1 = 10;
    }

    // Nạp âm: với cặp (diaChi 1..12, thienCan 1..10).
    let hanh = nap_am(menh_branch, HeavenlyStem::from_index((can_thang_menh_1 - 1) as u8).unwrap());

    // Map Hành → Cục:
    let (so, hanh_cuc) = match hanh {
        NguHanh::Kim => (4u8, NguHanh::Kim),
        NguHanh::Moc => (3, NguHanh::Moc),
        NguHanh::Thuy => (2, NguHanh::Thuy),
        NguHanh::Hoa => (6, NguHanh::Hoa),
        NguHanh::Tho => (5, NguHanh::Tho),
    };

    Cuc { so, hanh: hanh_cuc }
}

/// Bảng nạp âm Can-Chi → Hành. Lấy trực tiếp từ `lasotuvi.AmDuong.nguHanhNapAm`
/// (`matranNapAm`), 0-based trong Rust.
///
/// Quy ước index: `diaChi 0..11` (Tý=0, Sửu=1, Dần=2, ..., Hợi=11),
/// `thienCan 0..9` (Giáp=0, Ất=1, ..., Quý=9).
///
/// Bảng có cấu trúc đặc biệt: các cặp Can-Chi đi theo cặp (Giáp Tý, Ất Sửu),
/// (Bính Dần, Đinh Mão), (Mậu Thìn, Kỷ Tỵ), (Canh Ngọ, Tân Mùi), (Nhâm Thân,
/// Quý Dậu) → 6 cặp đầu xuôi; rồi tiếp tục lặp (Giáp Tuất, Ất Hợi), (Bính Tý,
/// Đinh Sửu), ... với các nhóm nạp âm quay vòng.
///
/// Tuy nhiên để đơn giản và đúng, dùng bảng tra cố định 12×10 như lasotuvi:
///
/// Vị trí (chi_0, can_0) cho biết Hành:
///
/// ```text
/// Chi\Can  Giáp Ất  Bính Đinh Mậu Kỷ  Canh Tân Nhâm Quý
/// Tý       K   T   T   -   H   -   O   -   M   -
/// Sửu      -   K   -   T   -   H   -   O   -   M
/// Dần      T   -   H   -   O   -   M   -   K   -
/// Mão      -   T   -   H   -   O   -   M   -   K
/// Thìn     H   -   O   -   M   -   K   -   T   -
/// Tỵ       -   H   -   O   -   M   -   K   -   T
/// Ngọ      K   -   T   -   H   -   O   -   M   -
/// Mùi      -   K   -   T   -   H   -   O   -   M
/// Thân     T   -   H   -   O   -   M   -   K   -
/// Dậu      -   T   -   H   -   O   -   M   -   K
/// Tuất     H   -   O   -   M   -   K   -   T   -
/// Hợi      -   H   -   O   -   M   -   K   -   T
/// ```
fn nap_am(chi: EarthlyBranch, can: HeavenlyStem) -> NguHanh {
    // Bảng 12×10; entry trống (-) → vô định, nhưng mỗi cặp Can-Chi có cấu trúc
    // luân phiên Giáp+Ất → Bính+Đinh → ... nên vị trí (chi, can) có Can nhỏ
    // ứng cho chi chẵn luôn dùng cột Giáp/Ất/Bính...; thực ra mỗi cặp kế tiếp
    // dịch phải 1. Để đơn giản: nếu Can-Chi sai khác chẵn-lẻ → bỏ qua và dùng
    // cặp chính.
    //
    // Tinh thần full: mỗi cặp liên tiếp (Giáp-Tý, Ất-Sửu) → cùng nạp âm, tiếp
    // theo (Bính-Dần, Đinh-Mão) → cùng nạp âm. Vị trí hành hoán vị theo chu kỳ
    // 6 (mỗi bước tạo 2 cặp, 6 bước ban đầu lặp 1 vòng đầy đủ). Hành:=
    //   group 0: Hải Trung Kim (K), group 1: Giáng Hạ Thủy (T), group 2: Tích
    //   Lịch Hỏa (H), group 3: Bích Thượng Thổ (O), group 4: Tang Đố Mộc (M),
    //   group 5: [...] K, group 6: K → repeated cycle K T H O M K T H O M ...
    //
    // Cách lưu bảng trực tiếp cho an toàn:
    use NguHanh::*;
    // 12 rows × 10 cols; vị trí None có nghĩa là cặp này không trực tiếp tồn
    // tại trong vòng Can-Chi 60 năm (vì chỉ có 60 cặp trong 12×10 = 120 ô, nửa
    // ô trống do quy tắc Can-Chi ghép chẵn-lẻ). Nhưng lasotuvi chỉ tra ô có dữ
    // liệu; để port miễn tăng độ tin cậy, ta chir trả.any canopyta trong 60
    // cặp — về mặt tính Cục thì Can của tháng sinh luôn cùng tính chẵn-lẻ với
    // Chi nên luôn đi vào ô có dữ liệu.
    //
    // Ký tự Hành của từng ô:
    //   K = Kim, T = Thủy, H = Hỏa, O = Thổ, M = Mộc, '-' = ô không thể xảy ra.
    const NAP_AM: [[Option<NguHanh>; 10]; 12] = [
        // Tý row (chi=0): K _ T _ H _ O _ M _
        [Some(Kim), None, Some(Thuy), None, Some(Hoa), None, Some(Tho), None, Some(Moc), None],
        // Sửu (chi=1) — Ất nên tiếp Tý: cùng Kim
        [None, Some(Kim), None, Some(Thuy), None, Some(Hoa), None, Some(Tho), None, Some(Moc)],
        // Dần (chi=2): T _ H _ O _ M _ K _
        [Some(Thuy), None, Some(Hoa), None, Some(Tho), None, Some(Moc), None, Some(Kim), None],
        // Mão
        [None, Some(Thuy), None, Some(Hoa), None, Some(Tho), None, Some(Moc), None, Some(Kim)],
        // Thìn (chi=4): H _ O _ M _ K _ T _
        [Some(Hoa), None, Some(Tho), None, Some(Moc), None, Some(Kim), None, Some(Thuy), None],
        // Tỵ
        [None, Some(Hoa), None, Some(Tho), None, Some(Moc), None, Some(Kim), None, Some(Thuy)],
        // Ngọ (chi=6): — повторяем chi=0 pattern theo tuổi vì 60-year cycle:
        // Ngọ year rounds to "K _ T _ H _ O _ M _" (giống Tý year pattern)
        [Some(Kim), None, Some(Thuy), None, Some(Hoa), None, Some(Tho), None, Some(Moc), None],
        // Mùi (chi=7)
        [None, Some(Kim), None, Some(Thuy), None, Some(Hoa), None, Some(Tho), None, Some(Moc)],
        // Thân (chi=8): T _ H _ O _ M _ K _ (giống Dần pattern)
        [Some(Thuy), None, Some(Hoa), None, Some(Tho), None, Some(Moc), None, Some(Kim), None],
        // Dậu (chi=9)
        [None, Some(Thuy), None, Some(Hoa), None, Some(Tho), None, Some(Moc), None, Some(Kim)],
        // Tuất (chi=10): H _ O _ M _ K _ T _ (giống Thìn pattern)
        [Some(Hoa), None, Some(Tho), None, Some(Moc), None, Some(Kim), None, Some(Thuy), None],
        // Hợi (chi=11)
        [None, Some(Hoa), None, Some(Tho), None, Some(Moc), None, Some(Kim), None, Some(Thuy)],
    ];

    NAP_AM[chi.index() as usize][can.index() as usize]
        // Trong trường hợp Can-Chi không đối (xảy ra nếu port sai, không trong
        // đủ bộ 60 cặp libros) → fallback về Hoàng. Hiếm gặp trong thực tế.
        .unwrap_or(NguHanh::Tho)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuc_so_in_2_to_6() {
        for stem_i in 0..10u8 {
            for chi_i in 0..12u8 {
                let menh = EarthlyBranch::from_index(chi_i).unwrap();
                let stem = HeavenlyStem::from_index(stem_i).unwrap();
                let cuc = tinh_cuc(menh, stem);
                assert!((2..=6).contains(&cuc.so), "cuc.so {} at menh={:?} stem={:?}", cuc.so, menh, stem);
            }
        }
    }

    #[test]
    fn cuc_ten_is_hanh_plus_so() {
        let cuc = Cuc { so: 2, hanh: NguHanh::Thuy };
        assert_eq!(cuc.name_vn(), "Thủy nhị Cục");
        let cuc = Cuc { so: 6, hanh: NguHanh::Hoa };
        assert_eq!(cuc.name_vn(), "Hỏa lục Cục");
    }
}