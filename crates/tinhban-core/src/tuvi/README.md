# tinhban-core / tuvi — Engine Tử Vi Đẩu Số (Bắc Tông)

Engine lập lá số Tử Vi Đẩu Số theo **trường phái Bắc Tông** (phổ biến nhất tại
Việt Nam hiện nay), an 12 cung + 14 chính tinh + ~14 phụ tinh + vòng Trường
Sinh. Đây là hạ tầng cho giai đoạn 6 (UI hiển thị lá số) và `tinhban-db` (lưu
lá số vào hồ sơ).

## Trường phái áp dụng

**Bắc Tông**. Khác biệt nhỏ với Nam Phái ở vài quy tắc an một số sao phụ (đặc
biệt Hỏa-Linh, Đào Hoa). Quy tắc an sao trong module tham chiếu trực tiếp
[`lasotuvi`](https://github.com/doanguyen/lasotuvi) (`doanguyen/lasotuvi` — mã
nguồn mở Python, cùng cộng đồng dùng thuật toán Hồ Ngọc Đức), đảm bảo đồng nhất
trường phái với `lich.vn`, `tuvi.cohoc.net` và phần lớn ứng dụng Tử Vi trực
tuyến tại Việt Nam.

## Phạm vi đã implement

1. **12 cung (Thập Nhị Cung)** + Cung Thân theo quy tắc "âm dương thuận nghịch".
2. **Cục**: tính bằng nạp âm Can-Chi của tháng âm lịch chứa cung Mệnh.
3. **14 chính tinh** (Tử Vi tinh hệ + Thiên Phủ tinh hệ).
4. **14 phụ tinh** phổ biến:
   - Tả Phù, Hữu Bật (an theo tháng sinh)
   - Văn Xương, Văn Khúc (an theo giờ sinh)
   - Thiên Khôi, Thiên Việt (an theo Can năm sinh)
   - Lộc Tồn (an theo Can năm sinh)
   - Kình Dương, Đà La (an kề Lộc Tồn ±1 cung)
   - Hỏa Tinh, Linh Tinh (an theo Chi năm + giờ + giới tính × âm-dương năm)
   - Thiên Mã (an theo Chi năm, tam hợp cục)
   - Đào Hoa (an theo vòng Kiếp Sát — từ Thiên Mã +3+4)
5. **Vòng Trường Sinh** (12 sao: Trường Sinh / Mộc Dục / Quan Đới / Lâm Quan /
   Đế Vượng / Suy / Bệnh / Tử / Mộ / Tuyệt / Thai / Dưỡng), chiều "Dương nam /
   Âm nữ thuận; Âm nam / Dương nữ nghịch" (theo cụ Thiên Lương, sửa từ "Nam
   thuận Nữ nghịch" cũ).

## Cố ý CHƯA implement (để giai đoạn sau)

- **Tứ Hóa** (Hóa Lộc / Hóa Quyền / Hóa Khoa / Hóa Kỵ — theo Can năm).
- **Vòng Lưu Hà / Thiên Trù**, **vòng Thái Tuế**, **vòng Lộc Tồn mở rộng**
  (Lực Sĩ / Thanh Long / Tiểu Hao / ...).
- **Vòng Tuần / Triệt** (Triệt không / Tuần không).
- **Bộ hang trăm sao phụ**: Long Trì / Phượng Các / Tam Thai / Bát Tọa / Ân
  Quang / Thiên Quý / Cô Thần / Quả Tú / Thiên Hình / Thiên Riêu / Kiếp Sát /
  Hoa Cái / Địa Không / Địa Kiếp / Hồng Loan / Thiên Hỷ / Thiên Quan / Thiên
  Phúc / Thai Phụ / Phong Cáo / Đẩu Quân / Lưu Hà / Thiên Trù.
- **Đại Hạn / Tiểu Hạn** (lasotuvi có `nhapDaiHan`/`nhapTieuHan` nhưng phase 3
  không tiêu chí nhất thiết phải có).

Module này cung cấp đủ để một lá số "dùng được" cơ bản: 14 chính tinh để soi
Mệnh / 3 hợp / cung chính, phụ tinh đủ để luận Lộc / Kỵ / Khôi-Việt / Hỏa-Linh /
Kình-Đà / Mã / Đào — chưa đủ để luận hoá đa tầng hoặc đại hạn.

## Public API

```rust
use tinhban_core::{lap_la_so, BirthMoment, Gender, Sao, EarthlyBranch};
use chrono::NaiveDate;

let birth = BirthMoment {
    solar_date: NaiveDate::from_ymd_opt(1991, 10, 24).unwrap(),
    hour: 7,        // wall-clock 0..23 (giờ Dương lịch)
    minute: 30,
};
let chart = lap_la_so(birth, Gender::Nam).unwrap();

// Cung Mệnh / Thân
let menh = chart.menh();                  // Palace
let than = chart.than();                   // Palace
let menh_branch = chart.menh_branch;      // EarthlyBranch (index)

// 12 cung:
for palace in chart.palaces.iter() {
    println!("{} ({:?}): {:?}", palace.name, palace.branch,
        palace.stars.iter().map(|s| s.name_vn()).collect::<Vec<_>>());
}

// Cục
println!("Cục: {} (số {})", chart.cuc, chart.cuc.so);

// Vòng Trường Sinh tại 1 cung
for s in chart.menh().truong_sinh.iter() {
    println!("Trường Sinh state at Mệnh: {}", s.name_vn());
}

// Tra một sao theo chi cung
let tuvi_branch = chart.palaces.iter()
    .find(|p| p.has_star(Sao::TuVi))
    .map(|p| p.branch)
    .unwrap();
```

## Quy trình đối chiếu (5 lá số mẫu × 2 giới tính = 10 case)

### Nguồn tham chiếu

**Chính:** `doanguyen/lasotuvi` (commit latest tại thời điểm phase 3 —
clone về `/tmp/opencode/tuvi_ref/lasotuvi/` và sinh reference bằng script
`gen_ref.py` / inline `gen_test_data` được đính trong script này).

**Phụ:** `lich.vn`, `tuvi.cohoc.net` (tham chiếu Web, không dùng trực tiếp làm
test vector — chỉ để xác nhận institutional Boca face).

### 5 lá số mẫu

Tất cả dùng wall-clock hour (semantic giống `BirthMoment.hour` 0..23).

| Case  | Dương lịch  | Giờ  | Giới | Âm lịch              |
|-------|-------------|------|------|----------------------|
| case1 | 24/10/1991  | 7    | Nam + Nữ | 17/9/1991 Tân Mùi |
| case2 | 17/02/2026  | 12   | Nam + Nữ | 1/1/2026 Bính Ngọ  |
| case3 | 10/02/2024  | 0    | Nam + Nữ | 1/1/2024 Giáp Thìn |
| case4 | 29/01/1990  | 9    | Nam + Nữ | 3/1/1990 Canh Ngọ  |
| case5 | 05/05/2000  | 15   | Nam + Nữ | 2/4/2000 Canh Thìn |

Mỗi case đối chiếu **27 vị trí sao** (14 chính tinh + 14 phụ tinh — gồm Tả
Phù / Hữu Bật / Văn Xương / Văn Khúc / Thiên Khôi / Thiên Việt / Lộc Tồn /
Kình Dương / Đà La / Hỏa Tinh / Linh Tinh / Thiên Mã / Đào Hoa) + cung
Mệnh / Thân + lunar date. Tổng cộng **10 × 27 = 270 vị trí sao** + 20 mốc
Mệnh/Thân + 10 mốc âm lịch = **300 đối chiếu**.

### Kết quả đối chiếu

**Tất cả pass: 270/270 sao positions, 10/10 menh/than correct, 10/10 lunar
date matches.**

```
running 4 tests
test each_palace_has_a_name ... ok
test sao_total_at_least_27_for_all_cases ... ok
test kinh_da_la_around_loc_ton ... ok
test all_10_reference_cases_match_lasotuvi ... ok
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Lỗi tìm thấy và đã sửa trong quá trình port

Trong khi implement, 3 lỗi tiềm tàng đã tự bộc lộ qua đối chiếu lasotuvi:

1. **Văn Xương ↔ Văn Khúc nhầm đổi**: Ho's code đặt Văn **KHÚC** tại
   Thìn chiều thuận theo giờ (dichCung(Thìn=5, gioSinh-1)), Văn **XƯƠNG** đối
   xứng qua trục Sửu-Mùi (dichCung(Sửu=2, 2 - VanKhuc)). Phiên bản đầu của
   module gán ngược 2 sao này. Sửa trong `stars/phu_tinh.rs`.
2. **Thiên Việt sai 1 cung**: công thức 0-based phải là `(9 - khoi_1) rem 12`
   (không phải `(8 - khoi_1)`) — phát hiện khi đối chiếu cho các năm có Can
   khác Giáp/Ất.
3. **Hỏa Tinh / Linh Tinh sai chi năm**: `tim_hoa_linh(chi_year_1, ...)` cần
   nhận chi năm 1-based, không phải 0-based. Ban đầu tôi truyền
   `year_branch_idx` (0-based) thẳng vào — kết quả lệch 1 nhóm tam hợp.

Đây chính là giá trị của việc đối chiếu với source uy tín: bắt được lỗiorpor
transcription mà các test nội bộ (như "Kình Đà kề Lộc Tồn") không phát hiện.

## Bản đồ module `tuvi/`

```
tuvi/
├── mod.rs             entry point `lap_la_so`, helper `dich_cung_i64`, từ lib.rs re-export
├── types.rs           `Gender`, `TuViChart`, `TuViError`
├── palaces.rs         `PalaceName` (enum 12 cung), `Palace`, `dich_cung`
├── cuc.rs             `Cuc`, `CucInfo`, `tinh_cuc`, bảng nạp âm 12x10
├── truong_sinh.rs     `TruongSinhState`, `truong_sinh_positions`
└── stars/
    ├── mod.rs          `Sao` enum (28 variants), `SaoCategory`, `name_vn`
    ├── chinh_tinh.rs   `an_chinh_tinh` — 14 chính tinh placement
    └── phu_tinh.rs     `an_phu_tinh` — 14 phụ tinh placement + `tim_hoa_linh`
```

## Giới hạn và known issues

- **Tử Vi & Thiên Phủ trùng cung**: theo quy ước Ho, khi Tử Vi rơi vào một số
  vị trí (e.g. Dần), Thiên Phủ (port từ `(4 - tuvi_idx) mod 12`) nằm cùng cung
  Dần. Đây là **đặc tính an sao**, không phải bug. Test
  `tu_vi_never_collides_with_thien_phu` đã được loại bỏ vì không đúng ý.
- **Chiều thuận nghịch của Trường Sinh**: Ho's code có hai phiên bản — bản gốc
  "Nam thuận Nữ nghịch" và bản sửa "Dương nam / Âm nữ thuận, Âm nam / Dương nữ
  nghịch" (theo cụ Thiên Lương). Module dùng bản sửa.
- **Phụ tinh giới**: Hỏa-Linh phụ thuộc `gioiTinh * amDuongNamSinh`. zasada
  Giới tính encode: Nam=+1, Nữ=-1; âm-dương năm theo Can năm: Dương = Giáp /
  Bính / Mậu / Canh / Nhâm; Âm = còn lại. Module theo lasotuvi.
- **Hiện chưa có** test cho tygodlimactリング Trường Sinh vì lasotuvi không
  expose trực tiếp dễ trích — chỉ có test nội bộ `thuan_visits_all_12_branches`
  kiểm tra tính nhất quán của vòng. Nếu phát hiện sai lệch với nguồn online,
  sẽ bổ sung sau.

## Tương lai (giai đoạn sau)

- **Tứ hóa** + vòng Lộc Tồn mở rộng — để giai đoạn tới khi bắt đầu luận giải
  sâu, hoặc khi tinhban-api cần expose Tứ Hóa cho UI.
- **Đại Hạn / Tiểu Hạn** — để giai đoạn 6 (UI hiển thị lá số + đại hạn).
- **Bộ sao phụ còn lại** — bổ sung từng nhóm khi cần (ưu tiên Long Trì / Phượng
  Các / Tam Thai / Bát Tọa / Hồng Loan / Thiên Hỷ vì hay gặp trong luận giải).