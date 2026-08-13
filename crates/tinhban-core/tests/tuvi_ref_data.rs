// =============================================================
// Tự sinh từ lasotuvi bằng gen_ref.py + convert_to_rust.py
// Lasotuvi commit: 1-based chi indices; chuyển về 0-based
// (Tý=0, Sửu=1, Dần=2, ..., Hợi=11) cho `EarthlyBranch::index()`.
//
// 5 ngày sinh × 2 giới tính = 10 case. Các mốc:
//   case1: 24/10/1991 giờ 7, nam & nữ (lunar 17/9/1991 Tân Mùi)
//   case2: 17/02/2026 giờ 12, nam & nữ (lunar 1/1/2026 Bính Ngọ)
//   case3: 10/02/2024 giờ 0, nam & nữ (lunar 1/1/2024 Giáp Thìn)
//   case4: 29/01/1990 giờ 9, nam & nữ (lunar 3/1/1990 Canh Ngọ)
//   case5: 05/05/2000 giờ 15, nam & nữ (lunar 2/4/2000 Canh Thìn)
//
// Lasotuvi uses wall-clock hour converted via gioSinh = ((hour+1)//2)%12+1.
// =============================================================

use tinhban_core::Gender;

pub struct RefCase {
    pub label: &'static str,
    pub birth: (i32, u32, u32, u8),   // year, month, day, hour
    pub gender: Gender,
    pub lunar_day: u8, pub lunar_month: u8, pub lunar_year: i32,
    pub menh_branch: u8,    // 0-based EarthlyBranch index
    pub than_branch: u8,    // 0-based
    pub sao_positions: &'static [(&'static str, u8)],  // (sao_name, 0-based chi)
}

pub const REF_CASES: &[RefCase] = &[
    RefCase {
        label: "case1_nam",
        birth: (1991, 10, 24, 7),
        gender: Gender::Nam,
        lunar_day: 17, lunar_month: 9, lunar_year: 1991,
        menh_branch: 6,
        than_branch: 2,
        sao_positions: &[("CuMon", 4), ("DaLa", 8), ("DaoHoa", 0), ("HoaTinh", 5), ("HuuBat", 2), ("KinhDuong", 10), ("LiemTrinh", 7), ("LinhTinh", 2), ("LocTon", 9), ("PhaQuan", 11), ("TaPhu", 0), ("ThaiAm", 2), ("ThaiDuong", 0), ("ThamLang", 3), ("ThatSat", 7), ("ThienCo", 2), ("ThienDong", 10), ("ThienKhoi", 6), ("ThienLuong", 6), ("ThienMa", 5), ("ThienPhu", 1), ("ThienTuong", 5), ("ThienViet", 2), ("TuVi", 3), ("VanKhuc", 8), ("VanXuong", 6), ("VuKhuc", 11)],
    },
    RefCase {
        label: "case1_nu",
        birth: (1991, 10, 24, 7),
        gender: Gender::Nu,
        lunar_day: 17, lunar_month: 9, lunar_year: 1991,
        menh_branch: 6,
        than_branch: 2,
        sao_positions: &[("CuMon", 4), ("DaLa", 8), ("DaoHoa", 0), ("HoaTinh", 1), ("HuuBat", 2), ("KinhDuong", 10), ("LiemTrinh", 7), ("LinhTinh", 6), ("LocTon", 9), ("PhaQuan", 11), ("TaPhu", 0), ("ThaiAm", 2), ("ThaiDuong", 0), ("ThamLang", 3), ("ThatSat", 7), ("ThienCo", 2), ("ThienDong", 10), ("ThienKhoi", 6), ("ThienLuong", 6), ("ThienMa", 5), ("ThienPhu", 1), ("ThienTuong", 5), ("ThienViet", 2), ("TuVi", 3), ("VanKhuc", 8), ("VanXuong", 6), ("VuKhuc", 11)],
    },
    RefCase {
        label: "case2_nam",
        birth: (2026, 2, 17, 12),
        gender: Gender::Nam,
        lunar_day: 1, lunar_month: 1, lunar_year: 2026,
        menh_branch: 8,
        than_branch: 8,
        sao_positions: &[("CuMon", 10), ("DaLa", 4), ("DaoHoa", 3), ("HoaTinh", 7), ("HuuBat", 10), ("KinhDuong", 6), ("LiemTrinh", 1), ("LinhTinh", 9), ("LocTon", 5), ("PhaQuan", 5), ("TaPhu", 4), ("ThaiAm", 8), ("ThaiDuong", 6), ("ThamLang", 9), ("ThatSat", 1), ("ThienCo", 8), ("ThienDong", 4), ("ThienKhoi", 11), ("ThienLuong", 0), ("ThienMa", 8), ("ThienPhu", 7), ("ThienTuong", 11), ("ThienViet", 9), ("TuVi", 9), ("VanKhuc", 10), ("VanXuong", 4), ("VuKhuc", 5)],
    },
    RefCase {
        label: "case2_nu",
        birth: (2026, 2, 17, 12),
        gender: Gender::Nu,
        lunar_day: 1, lunar_month: 1, lunar_year: 2026,
        menh_branch: 8,
        than_branch: 8,
        sao_positions: &[("CuMon", 10), ("DaLa", 4), ("DaoHoa", 3), ("HoaTinh", 7), ("HuuBat", 10), ("KinhDuong", 6), ("LiemTrinh", 1), ("LinhTinh", 9), ("LocTon", 5), ("PhaQuan", 5), ("TaPhu", 4), ("ThaiAm", 8), ("ThaiDuong", 6), ("ThamLang", 9), ("ThatSat", 1), ("ThienCo", 8), ("ThienDong", 4), ("ThienKhoi", 11), ("ThienLuong", 0), ("ThienMa", 8), ("ThienPhu", 7), ("ThienTuong", 11), ("ThienViet", 9), ("TuVi", 9), ("VanKhuc", 10), ("VanXuong", 4), ("VuKhuc", 5)],
    },
    RefCase {
        label: "case3_nam",
        birth: (2024, 2, 10, 0),
        gender: Gender::Nam,
        lunar_day: 1, lunar_month: 1, lunar_year: 2024,
        menh_branch: 2,
        than_branch: 2,
        sao_positions: &[("CuMon", 10), ("DaLa", 1), ("DaoHoa", 9), ("HoaTinh", 2), ("HuuBat", 10), ("KinhDuong", 3), ("LiemTrinh", 1), ("LinhTinh", 10), ("LocTon", 2), ("PhaQuan", 5), ("TaPhu", 4), ("ThaiAm", 8), ("ThaiDuong", 6), ("ThamLang", 9), ("ThatSat", 1), ("ThienCo", 8), ("ThienDong", 4), ("ThienKhoi", 1), ("ThienLuong", 0), ("ThienMa", 2), ("ThienPhu", 7), ("ThienTuong", 11), ("ThienViet", 7), ("TuVi", 9), ("VanKhuc", 4), ("VanXuong", 10), ("VuKhuc", 5)],
    },
    RefCase {
        label: "case3_nu",
        birth: (2024, 2, 10, 0),
        gender: Gender::Nu,
        lunar_day: 1, lunar_month: 1, lunar_year: 2024,
        menh_branch: 2,
        than_branch: 2,
        sao_positions: &[("CuMon", 10), ("DaLa", 1), ("DaoHoa", 9), ("HoaTinh", 2), ("HuuBat", 10), ("KinhDuong", 3), ("LiemTrinh", 1), ("LinhTinh", 10), ("LocTon", 2), ("PhaQuan", 5), ("TaPhu", 4), ("ThaiAm", 8), ("ThaiDuong", 6), ("ThamLang", 9), ("ThatSat", 1), ("ThienCo", 8), ("ThienDong", 4), ("ThienKhoi", 1), ("ThienLuong", 0), ("ThienMa", 2), ("ThienPhu", 7), ("ThienTuong", 11), ("ThienViet", 7), ("TuVi", 9), ("VanKhuc", 4), ("VanXuong", 10), ("VuKhuc", 5)],
    },
    RefCase {
        label: "case4_nam",
        birth: (1990, 1, 29, 9),
        gender: Gender::Nam,
        lunar_day: 3, lunar_month: 1, lunar_year: 1990,
        menh_branch: 9,
        than_branch: 7,
        sao_positions: &[("CuMon", 5), ("DaLa", 7), ("DaoHoa", 3), ("HoaTinh", 6), ("HuuBat", 10), ("KinhDuong", 9), ("LiemTrinh", 6), ("LinhTinh", 10), ("LocTon", 8), ("PhaQuan", 0), ("TaPhu", 4), ("ThaiAm", 3), ("ThaiDuong", 11), ("ThamLang", 4), ("ThatSat", 8), ("ThienCo", 1), ("ThienDong", 9), ("ThienKhoi", 7), ("ThienLuong", 7), ("ThienMa", 8), ("ThienPhu", 2), ("ThienTuong", 6), ("ThienViet", 1), ("TuVi", 2), ("VanKhuc", 9), ("VanXuong", 5), ("VuKhuc", 10)],
    },
    RefCase {
        label: "case4_nu",
        birth: (1990, 1, 29, 9),
        gender: Gender::Nu,
        lunar_day: 3, lunar_month: 1, lunar_year: 1990,
        menh_branch: 9,
        than_branch: 7,
        sao_positions: &[("CuMon", 5), ("DaLa", 7), ("DaoHoa", 3), ("HoaTinh", 8), ("HuuBat", 10), ("KinhDuong", 9), ("LiemTrinh", 6), ("LinhTinh", 8), ("LocTon", 8), ("PhaQuan", 0), ("TaPhu", 4), ("ThaiAm", 3), ("ThaiDuong", 11), ("ThamLang", 4), ("ThatSat", 8), ("ThienCo", 1), ("ThienDong", 9), ("ThienKhoi", 7), ("ThienLuong", 7), ("ThienMa", 8), ("ThienPhu", 2), ("ThienTuong", 6), ("ThienViet", 1), ("TuVi", 2), ("VanKhuc", 9), ("VanXuong", 5), ("VuKhuc", 10)],
    },
    RefCase {
        label: "case5_nam",
        birth: (2000, 5, 5, 15),
        gender: Gender::Nam,
        lunar_day: 2, lunar_month: 4, lunar_year: 2000,
        menh_branch: 9,
        than_branch: 1,
        sao_positions: &[("CuMon", 5), ("DaLa", 7), ("DaoHoa", 9), ("HoaTinh", 10), ("HuuBat", 7), ("KinhDuong", 9), ("LiemTrinh", 6), ("LinhTinh", 2), ("LocTon", 8), ("PhaQuan", 0), ("TaPhu", 7), ("ThaiAm", 3), ("ThaiDuong", 11), ("ThamLang", 4), ("ThatSat", 8), ("ThienCo", 1), ("ThienDong", 9), ("ThienKhoi", 7), ("ThienLuong", 7), ("ThienMa", 2), ("ThienPhu", 2), ("ThienTuong", 6), ("ThienViet", 1), ("TuVi", 2), ("VanKhuc", 0), ("VanXuong", 2), ("VuKhuc", 10)],
    },
    RefCase {
        label: "case5_nu",
        birth: (2000, 5, 5, 15),
        gender: Gender::Nu,
        lunar_day: 2, lunar_month: 4, lunar_year: 2000,
        menh_branch: 9,
        than_branch: 1,
        sao_positions: &[("CuMon", 5), ("DaLa", 7), ("DaoHoa", 9), ("HoaTinh", 6), ("HuuBat", 7), ("KinhDuong", 9), ("LiemTrinh", 6), ("LinhTinh", 6), ("LocTon", 8), ("PhaQuan", 0), ("TaPhu", 7), ("ThaiAm", 3), ("ThaiDuong", 11), ("ThamLang", 4), ("ThatSat", 8), ("ThienCo", 1), ("ThienDong", 9), ("ThienKhoi", 7), ("ThienLuong", 7), ("ThienMa", 2), ("ThienPhu", 2), ("ThienTuong", 6), ("ThienViet", 1), ("TuVi", 2), ("VanKhuc", 0), ("VanXuong", 2), ("VuKhuc", 10)],
    },
];
