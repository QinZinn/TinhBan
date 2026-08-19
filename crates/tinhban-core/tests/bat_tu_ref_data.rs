// =============================================================
// Tự sinh từ Ho's algorithm bằng gen_bat_tu_ref.py + Python prototype.
// Convention: tiet_khi JD = calendar day CONTAINING the transition (jd-1).
// Index 0-based: Can Giáp=0..Quý=9, Chi Tý=0..Hợi=11.
// =============================================================

#[allow(dead_code)]
pub struct RefBtCase {
    pub label: &'static str,
    pub birth: (i32, u32, u32, u8),   // year, month, day, hour
    pub bt_year: i32,
    pub bt_month_1: u8,    // 1..12 (Dần..Sửu)
    pub year_can_idx: u8, pub year_chi_idx: u8,
    pub month_can_idx: u8, pub month_chi_idx: u8,
    pub day_can_idx: u8, pub day_chi_idx: u8,
    pub hour_can_idx: u8, pub hour_chi_idx: u8,
}

pub const REF_BT_CASES: &[RefBtCase] = &[
    RefBtCase {
        label: "case1",
        birth: (1991, 10, 24, 7),
        bt_year: 1991,
        bt_month_1: 9,
        year_can_idx: 7, year_chi_idx: 7,
        month_can_idx: 4, month_chi_idx: 10,
        day_can_idx: 3, day_chi_idx: 3,
        hour_can_idx: 0, hour_chi_idx: 4,
    },
    RefBtCase {
        label: "case2",
        birth: (2026, 2, 17, 12),
        bt_year: 2026,
        bt_month_1: 1,
        year_can_idx: 2, year_chi_idx: 6,
        month_can_idx: 6, month_chi_idx: 2,
        day_can_idx: 8, day_chi_idx: 10,
        hour_can_idx: 2, hour_chi_idx: 6,
    },
    RefBtCase {
        label: "case3",
        birth: (2024, 2, 10, 0),
        bt_year: 2024,
        bt_month_1: 1,
        year_can_idx: 0, year_chi_idx: 4,
        month_can_idx: 2, month_chi_idx: 2,
        day_can_idx: 0, day_chi_idx: 4,
        hour_can_idx: 0, hour_chi_idx: 0,
    },
    RefBtCase {
        label: "case4",
        birth: (1990, 1, 29, 9),
        bt_year: 1989,
        bt_month_1: 12,
        year_can_idx: 5, year_chi_idx: 5,
        month_can_idx: 3, month_chi_idx: 1,
        day_can_idx: 0, day_chi_idx: 6,
        hour_can_idx: 5, hour_chi_idx: 5,
    },
    RefBtCase {
        label: "case5",
        birth: (2000, 5, 5, 15),
        bt_year: 2000,
        bt_month_1: 4,
        year_can_idx: 6, year_chi_idx: 4,
        month_can_idx: 7, month_chi_idx: 5,
        day_can_idx: 9, day_chi_idx: 11,
        hour_can_idx: 6, hour_chi_idx: 8,
    },
    RefBtCase {
        label: "case6",
        birth: (1991, 2, 6, 14),
        bt_year: 1991,
        bt_month_1: 1,
        year_can_idx: 7, year_chi_idx: 7,
        month_can_idx: 6, month_chi_idx: 2,
        day_can_idx: 3, day_chi_idx: 7,
        hour_can_idx: 3, hour_chi_idx: 7,
    },
    RefBtCase {
        label: "case7",
        birth: (1990, 2, 1, 8),
        bt_year: 1989,
        bt_month_1: 12,
        year_can_idx: 5, year_chi_idx: 5,
        month_can_idx: 3, month_chi_idx: 1,
        day_can_idx: 3, day_chi_idx: 9,
        hour_can_idx: 0, hour_chi_idx: 4,
    },
];
