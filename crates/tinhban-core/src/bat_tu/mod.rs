//! Engine lập lá số Bát Tự (Tứ Trụ) — Bát Tự engine.
//!
//! # Phạm vi đã implement (đã chốt với spec giai đoạn 4)
//!
//! 1. **Tứ Trụ** (8 chữ: Can+Chi của năm/tháng/ngày/giờ):
//!    - Trụ Năm: dùng `year_can_chi` từ phase 2, nhưng **năm Bát Tự** tính
//!      theo **tiết khí Lập Xuân** (315°) — sinh trước Lập Xuân (khoảng 4-5/2
//!      Dương lịch) thì trụ Năm thuộc năm Can Chi cũ; sinh sau Lập Xuân thì
//!      thuộc năm Can Chi năm đó. Khác với Tử Vi/lịch âm thường (năm đổi theo
//!      Tết Nguyên Đán). Chức năng tiết khí được cung cấp bởi module con
//!      [`tiet_khi`] (dùng công thức trời văn Hồ Ngọc Đức đã có sẵn từ phase 2).
//!    - Trụ Tháng: dùng `month_can_chi` từ phase 2 với **tháng Bát Tự** (= tháng
//!      tiết khí, KHÔNG phải tháng âm lịch Tử Vi). Module [`tiet_khi`] xác định
//!      tháng tiết khí dựa vào 12 tiết (Lập Xuân/Kinh Trập/.../Tiểu Hàn) —
//!      xem bảng trong [`tiet_khi::TIET_KHI_TABLE`].
//!    - Trụ Ngày: dùng `day_can_chi` (Julian Day công thức chuẩn `(jd+9)%10`,
//!      `(jd+1)%12` — không khác biệt với Can Chi ngày thường).
//!    - Trụ Giờ: dùng `hour_can_chi` (12 giờ Địa Chi truyền thống, giờ Tý=
//!      23h-1h, theo "Ngũ Thử Độn Thời").
//! 2. **Thống kê Ngũ Hành** trong 8 chữ (4 Can + 4 Chi). Output `NguHanhCount`:
//!    chỉ đếm **Hành chính** của mỗi Can / Chi (không tính Tàng Can).
//! 3. **Thập Thần** (Ten Gods) cho 7 Can còn lại (của trụ Năm/Tháng/Giờ, không
//!    tính Can ngày = Nhật Chủ) VÀ **Tàng Can** của 4 Chi (mỗi Chi có 1-3 Can
//!    ẩn theo bảng cố định trong [`hidden_stems`]).
//!
//! # Cố ý CHƯA implement (để giai đoạn sau)
//!
//! - **Vượng / Suy Nhật Chủ** (luận ngày được / không được mùa) — nhiều trường
//!   phái tranh cãi, cần luận giải sâu hơn, để sau.
//! - **Dụng Thần / Kỵ Thần** — chọn hành cần bổ sung / tránh, phụ thuộc luận
//!   Vượng-Suy + cấu trúc lá số, để sau.
//! - **Đại Vận** (Luck Pillars — vận trình 10 năm): tính dựa theo giới tính ×
//!   Can năm × chiều thuận nghịch + khoảng cách tới tiết khí gần nhất. Để sau.
//! - **Lưu Niên** (vận từng năm) — để sau.
//! - **UI hiển thị lá số Bát Tự** — giai đoạn 6.
//!
//! # Giới hạn tiết khí (±1 ngày)
//!
//! Ho's polynomial formula cho kinh độ Mặt Trời có độ chính xác ~±0.5° so với
//! high-precision ephemeris (NASA / HK Observatory). Vì Sun tiến ~0.985°/ngày,
//! ±0.5° ở kinh độ ≈ ±0.5 ngày ở thời điểm tiết khí. Khuyến nghị:
//! ngày sinh **cách biên tiết ≥ 5 ngày** để BT year/month được xác định
//! unambiguously. Lá số ở giữa tháng BT không bị ảnh hưởng.
//!
//! # Convention hour boundary
//!
//! Trụ Ngày dùng Julian Day của birth_date (calendar day tại UTC+7). Sinh ở
//! 23:30 hôm nay theo convention này vẫn thuộc **nay** (không phải "giờ Tý của
//! ngày mai"). Một số trường phái Bát Tự cổ truyền् dùng convention "giờ Tý
//! (23:00) đánh dấu ngày mới" → lịch's khác cách 1 ngày; module này theo
//! convention lasotuvi / hiện đại cho nhất quán với phase 2 `hour_can_chi`.

pub mod hidden_stems;
pub mod thap_than;
pub mod tiet_khi;
pub mod types;

pub use hidden_stems::hidden_stems;
pub use thap_than::{TenGod, ten_god_of};
pub use tiet_khi::{
    TIET_KHI_24, TIET_KHI_TABLE, current_tiet_khi, find_tiet_khi_jd, lap_xuan_jd,
    tiet_khi_jds_of_bt_year, tiet_month_branch_index, tiet_month_index,
};
pub use types::{BatTuChart, BatTuError, Gender, NguHanhCount, Pillar};

use crate::{
    day_can_chi, hour_can_chi, month_can_chi, nguhanh_of_branch, nguhanh_of_stem,
    year_can_chi, BirthMoment, CanChi, EarthlyBranch, HeavenlyStem, NguHanh,
};
use chrono::Datelike;

/// Lập lá số Bát Tự từ ngày giờ sinh + giới tính.
pub fn lap_bat_tu(birth: BirthMoment, gender: Gender) -> Result<BatTuChart, BatTuError> {
    check_range(birth.solar_date.year())?;

    let birth_jd = crate::astronomy::jd_from_date(
        birth.solar_date.day() as i64,
        birth.solar_date.month() as i64,
        birth.solar_date.year() as i64,
    );

    // 1. Xác định năm Bát Tự (BT year) theo Lập Xuân.
    let lap_xuan_curr = lap_xuan_jd(birth.solar_date.year());
    let lap_xuan_next = lap_xuan_jd(birth.solar_date.year() + 1);

    let bt_year = if birth_jd < lap_xuan_curr {
        birth.solar_date.year() - 1
    } else if birth_jd >= lap_xuan_next {
        birth.solar_date.year() + 1
    } else {
        birth.solar_date.year()
    };

    let year_cc = year_can_chi(bt_year)?;

    // 2. Xác định tháng Bát Tự (theo 12 tiết).
    let tiet_jds = tiet_khi_jds_of_bt_year(bt_year);
    // find smallest i such that birth_jd >= tiet_jds[i] AND birth_jd < next
    let bt_month_idx_0 = tiet_jds
        .iter()
        .enumerate()
        .rev()
        .find(|&(_, &jd_t)| birth_jd >= jd_t)
        .map(|(i, _)| i)
        .unwrap_or_else(|| {
            // Edge case: sinh trước cả tiết đầu (Lập Xuân) → vẫn thuộc tháng Dần
            // của năm BT trước đó. Tuy logic branch này理论上不可能 xảy ra vì
            // bt_year đã chọn sao cho birth_jd >= lap_xuan_of_bt_year.
            0
        });
    // Determine tháng cuối (Sửu). If birth_jd > tiet_jds[11] (Tiểu Hàn của bt_year),
    // ngày vẫn trong tháng Sửu (tháng 12, idx_0 = 11).
    // Nếu kỳ lạ birth_jd đã >= lap_xuan_next thì bt_year sẽ tăng +1 (đã handle ở trên);
    // trong bt_year, tháng index 0..11; nếu birth_jd >= tiet_jds[11] thì tháng = 11
    // (Sửu) đến lap_xuan_next.
    let bt_month_idx_0 = u8::try_from(bt_month_idx_0).unwrap_or(0);
    let bt_month_1based = bt_month_idx_0 + 1; // 1..12

    // 3. Lập các trụ Can-Chi bằng hàm phase 2.
    let year_can = year_cc.stem;
    let month_cc = month_can_chi(year_can, bt_month_1based)?;
    let day_cc = day_can_chi(birth.solar_date)?;
    let hour_cc = hour_can_chi(day_cc.stem, birth.hour)?;

    // 4. Tính Thập Thần cho 3 trụ (Năm/Tháng/Giờ) + Tàng Can của 4 trụ.
    let nhat_chu = day_cc.stem;
    let year_stem_tg = ten_god_of(nhat_chu, year_cc.stem);
    let month_stem_tg = ten_god_of(nhat_chu, month_cc.stem);
    // Trụ Ngày: ten_god = None (vì chính là Nhật Chủ).
    let hour_stem_tg = ten_god_of(nhat_chu, hour_cc.stem);

    let year_hs = hidden_stems_with_tg(year_cc.branch, nhat_chu);
    let month_hs = hidden_stems_with_tg(month_cc.branch, nhat_chu);
    let day_hs = hidden_stems_with_tg(day_cc.branch, nhat_chu);
    let hour_hs = hidden_stems_with_tg(hour_cc.branch, nhat_chu);

    let year_pillar = Pillar {
        can_chi: year_cc,
        ten_god: Some(year_stem_tg),
        hidden_stems: year_hs,
    };
    let month_pillar = Pillar {
        can_chi: month_cc,
        ten_god: Some(month_stem_tg),
        hidden_stems: month_hs,
    };
    let day_pillar = Pillar {
        can_chi: day_cc,
        ten_god: None,
        hidden_stems: day_hs,
    };
    let hour_pillar = Pillar {
        can_chi: hour_cc,
        ten_god: Some(hour_stem_tg),
        hidden_stems: hour_hs,
    };

    // 5. Thống kê Ngũ Hành (Hành chính của 4 Can + 4 Chi).
    let pillars: [&CanChi; 4] = [&year_cc, &month_cc, &day_cc, &hour_cc];
    let mut count = NguHanhCount::default();
    for cc in pillars.iter() {
        increment_hanh(&mut count, nguhanh_of_stem(cc.stem));
        increment_hanh(&mut count, nguhanh_of_branch(cc.branch));
    }

    Ok(BatTuChart {
        birth,
        gender,
        year_pillar,
        month_pillar,
        day_pillar,
        hour_pillar,
        nguhanh_count: count,
    })
}

fn increment_hanh(c: &mut NguHanhCount, h: NguHanh) {
    match h {
        NguHanh::Kim => c.kim += 1,
        NguHanh::Moc => c.moc += 1,
        NguHanh::Thuy => c.thuy += 1,
        NguHanh::Hoa => c.hoa += 1,
        NguHanh::Tho => c.tho += 1,
    }
}

fn hidden_stems_with_tg(branch: EarthlyBranch, nhat_chu: HeavenlyStem) -> Vec<(HeavenlyStem, TenGod)> {
    hidden_stems(branch)
        .iter()
        .map(|&stem| (stem, ten_god_of(nhat_chu, stem)))
        .collect()
}

fn check_range(year: i32) -> Result<(), BatTuError> {
    if !(1900..=2100).contains(&year) {
        return Err(BatTuError::OutOfRange(format!(
            "năm {} ngoài phạm vi hỗ trợ 1900–2100",
            year
        )));
    }
    Ok(())
}

impl BatTuChart {
    /// Nhật Chủ = Can của trụ Ngày.
    pub fn nhat_chu(&self) -> HeavenlyStem {
        self.day_pillar.can_chi.stem
    }

    /// Liệt kê tất cả 8 chữ (4 Can + 4 Chi) theo thứ tự Năm/Tháng/Ngày/Giờ.
    pub fn all_8_chars(&self) -> [(HeavenlyStem, EarthlyBranch); 4] {
        [
            (self.year_pillar.stem(), self.year_pillar.branch()),
            (self.month_pillar.stem(), self.month_pillar.branch()),
            (self.day_pillar.stem(), self.day_pillar.branch()),
            (self.hour_pillar.stem(), self.hour_pillar.branch()),
        ]
    }
}

#[cfg(test)]
mod smoke {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn lap_bat_tu_smoke_does_not_panic_for_a_safe_birth() {
        // 24/10/1991 — mid-October, rất xa biên tiết.
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(1991, 10, 24).unwrap(),
            hour: 7,
            minute: 0,
        };
        let chart = lap_bat_tu(birth, Gender::Nam).unwrap();
        assert_eq!(
            (chart.year_pillar.stem(), chart.year_pillar.branch()),
            (HeavenlyStem::Tan, EarthlyBranch::Mui),
            "expected Tân Mùi for Oct 24 1991",
        );
        // Ngũ Hành total = 8 (4 Can + 4 Chi).
        assert_eq!(chart.nguhanh_count.total(), 8);
    }

    #[test]
    fn lap_bat_tu_year_prior_to_lap_xuan_returns_previous_year() {
        // 10/01/1990 (~4 weeks pre-Lập Xuân 1990) → BT year = 1989 (Kỷ Tỵ).
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(1990, 1, 10).unwrap(),
            hour: 10,
            minute: 0,
        };
        let chart = lap_bat_tu(birth, Gender::Nu).unwrap();
        assert_eq!(
            (chart.year_pillar.stem(), chart.year_pillar.branch()),
            (HeavenlyStem::Ky, EarthlyBranch::Ty2),
            "expected Kỷ Tỵ for Jan 10 1990 (pre-Lập Xuân 1990)",
        );
    }

    #[test]
    fn nguhanh_count_for_safe_chart_has_total_eight() {
        // Năm Tân Mùi: Tân=Kim, Mùi=Thổ → 2 chars count Kim+1 Thổ+1.
        // Tháng: October 24 → tháng BT Dần-Mão... actually Oct 24 in TL → tháng
        // Tuất. Detail không matter ở smoke level; chỉ kiểm tra tổng = 8.
        let birth = BirthMoment {
            solar_date: NaiveDate::from_ymd_opt(1991, 10, 24).unwrap(),
            hour: 7,
            minute: 0,
        };
        let chart = lap_bat_tu(birth, Gender::Nam).unwrap();
        assert_eq!(chart.nguhanh_count.total(), 8);
    }
}