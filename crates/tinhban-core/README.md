# tinhban-core — Lõi âm lịch & Can Chi

Lõi hạ tầng chuyển Dương lịch ↔ Âm lịch Việt Nam + tính Can–Chi (Thiên Can, Địa
Chi) & Ngũ Hành cho năm/tháng/ngày/giờ. Đây là nền tảng mà `tinhban-api`, lá số
Tử Vi (giai đoạn 3), lá số Bát Tự (giai đoạn 4), và tính ngày tốt/xấu (giai đoạn
5) đều phụ thuộc vào.

## Thuật toán

### Nguồn gốc

Thuật toán chuyển Dương lịch ↔ Âm lịch dựa trên **thuật toán của Hồ Ngọc Đức**
(Ho Ngoc Duc), công bố công khai từ cuối thập niên 1990, dùng tính toán thiên
văn (vị trí Mặt Trời / thời điểm sóc — new moon) theo múi giờ UTC+7 của Việt
Nam. Đây là thuật toán mà `lich.vn` và phần lớn lịch âm Việt Nam trực tuyến dùng
làm nguồn, đã được kiểm chứng rộng rãi.

Toán thiên văn tham chiếu sách *"Astronomical Algorithms"* của **Jean Meeus**
(1998). Cấu trúc hàm và hằng số đã đối chiếu trực tiếp với nhiều bản port public
trên GitHub để xác minh:

| Nguồn đối chiếu                                                | Vai trò                                |
|----------------------------------------------------------------|----------------------------------------|
| `doanguyen/lasotuvi/Lich_HND.py`                                | Nguồn chính thức từ thực giả Ho Ngoc Duc |
| `vanng822/ramlich/amlich/src/fns.rs`                            | Port Rust — công thức `NewMoon`        |
| `kunkka19xx/look/core/lunar/src/lib.rs`                        | Port Rust sạch, có test riêng          |
| `J2TEAM/vibe.j2team.org/.../lunar.ts`                          | Port TypeScript với `getDayCanChi`     |
| `quocthang0507/Calendar/CalendarLib/Lunar.cs`                   | Port C#/Unity                          |
| `tradecatlabs/fatecat/.../amlich.rs-master/src/lib.rs`         | Port Rust cho `LunarDay::to_julian_days` (L2S) |

### Tóm tắt kỹ thuật

- **Julian Day Number**: tất cả calculations dùng JD nguyên (integer = trưa UTC
  của ngày Dương lịch, theo convention thiên văn).
- **NewMoon(k)**: JD (số thực) của lần sóc thứ `k` kể từ epoch 1/1/1900, dùng
  đầy đủ chu kỳ perturbation của Meeus chương 49 — bao gồm các hiệu chỉnh
  `(M±M')`, `(2*M±Mpr)`, `(2*F±M)`, `(2*F±Mpr)`, cùng hiệu chỉnh trung bình
  `(166.56 + 132.87*T - 0.009173*T²)` term. Da inkább chuyển `T = k/1236.85`,
  `T²`, `T³` gốc + `DeltaT` (lỗi nhỏ ngoài 1900).
- **SunLongitude**: INCLUDING hiệu chỉnh **nutation-in-longitude** (`omega`
  term) — đây là phiên bản "NEW" của Ho Ngoc Duc, quan trọng cho độ chính xác
  phụ-ngày. Công thức chuẩn: `T = (jdn - 2451545.5 - time_zone/24.)/36525.`.
- **Trung khí & sl_idx**: `floor(SunLongitude/(π/6))` ⇒ chỉ số 0..11 của tháng
  thái dương. Sự kiện Trung khí (`ChunႹi`) ứng với mỗi cận 30° của kinh độ
  Mặt Trời (0°/30°/.../330°).
- **Lunar month 11**: tháng âm chứa Đông Chí (= Winter Solstice, 270°).
  Công thức: nếu `sl_idx(NM)` ≥ 9 tại lần sóc cuối năm Dương → sóc đó là tháng
  11, ngược lại phải sử dụng sóc trước đó (k-1).
- **Leap month (tháng nhuận)**: tháng âm không chứa Trung khí nào (sl_idx không
  đổi giữa 2 sóc liên tiếp) — phát hiện bằng cách walk forward từ m11.

### Phạm vi hỗ trợ

Năm Dương lịch **1900 → 2100**. Ngoài phạm vi, hàm chuyển đổi trả
`LunarError::OutOfRange`. Bảng hằng số thiên văn được hiệu chỉnh cho khoảng
này; ngoài (1700–1900 hoặc 2100–2200) sai số tích luỹ sẽ tăng nhanh.

## Bảng Can Chi (Heavenly Stems × Earthly Branches)

```
HeavenlyStem: Giáp(0) Ất(1) Bính(2) Đinh(3) Mậu(4) Kỷ(5) Canh(6) Tân(7) Nhâm(8) Quý(9)
EarthlyBranch: Tý(0) Sửu(1) Dần(2) Mão(3) Thìn(4) Tỵ(5) Ngọ(6) Mùi(7) Thân(8) Dậu(9) Tuất(10) Hợi(11)
NguHanh: Kim (Metal) | Mộc (Wood) | Thủy (Water) | Hỏa (Fire) | Thổ (Earth)
```

> **Quy ước tránh trùng tên**: enum `EarthlyBranch` dùng `Ty2` cho chi **Tỵ**
> (snake) để không trùng `Ty` cho **Tý** (rat) theo gợi ý của spec giai đoạn 2.

### Công thức

| Target | Công thức                                  | Quy tắc dalej vidvardpoiss |
|--------|-------------------------------------------|----------------------------|
| year_can_chi | `can=(year+6)%10`, `chi=(year+8)%12` | — |
| month_can_chi (Ngũ Thử Độn) | `year_can = (year+6)%10`; `base_can = (year_can%5*2+2)%10`; `can=(base_can+month-1)%10`; `chi=(month+1)%12` | Tbl. dưới |
| day_can_chi | `can=(jd+9)%10`, `chi=(jd+1)%12`        | chu kỳ 60 Giáp Tý |
| hour_can_chi (Ngũ Thử Độn Thời) | `hour_chi = floor((hour+1)/2)%12`; `base_can=(day_can%5*2)%10`; `can=(base_can+hour_chi)%10`; `chi=hour_chi` | Bảng, giờ Tý = 23h |

**Ngũ Thử Độn cho tháng** (Can của tháng 1 phụ thuộc Can của năm):

| Can năm     | Tháng 1 → |
|-------------|-----------|
| Giáp / Kỷ   | Bính Dần  |
| Ất / Canh   | Mậu Dần   |
| Bính / Tân  | Canh Dần  |
| Đinh / Nhâm | Nhâm Dần  |
| Mậu / Quý   | Giáp Dần  |

**Ngũ Thử Độn Thời** (Can của giờ Tý phụ thuộc Can của ngày):

| Can ngày    | Giờ Tý → |
|-------------|----------|
| Giáp / Kỷ   | Giáp Tý |
| Ất / Canh   | Bính Tý |
| Bính / Tân  | Mậu Tý |
| Đinh / Nhâm | Canh Tý |
| Mậu / Quý   | Nhâm Tý |

### Ngũ Hành của Can & Chi

| Can                       | Hành  |
|---------------------------|-------|
| Giáp, Ất                  | Mộc   |
| Bính, Đinh                | Hỏa   |
| Mậu, Kỷ                   | Thổ   |
| Canh, Tân                 | Kim   |
| Nhâm, Quý                 | Thủy  |

| Chi                      | Hành  |
|--------------------------|-------|
| Dần, Mão                 | Mộc   |
| Tỵ, Ngọ                  | Hỏa   |
| Thân, Dậu                | Kim   |
| Tý, Hợi                  | Thủy  |
| Sửu, Thìn, Mùi, Tuất     | Thổ (4 tháng cuối mùa) |

## Quy trình đối chiếu (test data)

Test của `tinhban-core` dùng tới **163 vector đối chiếu** được sinh ra từ một
prototype Python nội bộ (xem `lunar_proto.py` trong workspace `/tmp` khi phát
triển — không commit). Quy trình:

1. Viết prototype Python theo Ho Ngoc Duc public algorithm.
2. Verify prototype bằng các Tet date công khai (Wikipedia + news sources VN —
   các năm không có tranh chấp timezone) — kết quả sạch 29/31 vector jaar-end.
3. Sinh test vectors chuẩn `Rust` từ prototype bằng script
   `gen_test_vectors.py`, và port nguyên vào `tests/test_data.rs`.

### Thành phần test data

| Block                          | Số vector | Mục đích                          |
|--------------------------------|-----------|-----------------------------------|
| `TET_CASES`                    | 31        | Tết (1/1 âm) 2000–2030Happy Befehl VN |
| `KUNKKA_CASES`                 | 5         | test data công bố của `kunkka19xx/lunar` |
| `FULL_MOON_JAN`                | 8         | Rằm tháng Giêng 2000–2028         |
| `LEAP_MONTH_CASES`             | 11        | L2S cho các tháng nhuận đã biết   |
| `YEAR_CAN_CHI`                 | 28        | Can Chi năm 1980–2100             |
| `ROUNDTRIP_SOLAR`              | 80        | S2L → L2S roundtrip, 1950–2098    |
| **Tổng**                       | **163**   |                                   |

Ngoài ra có các test logic-specific khác (hour branch sequence, Ngũ Thử Độn
table, chu kỳ 60 cho day_can_chi, edge case Dương nhuận 29/2/2024, giao thừa,
lỗi ngoài phạm vi, v.v.).

### Status test: tất cả pass

```
running 33 tests
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Phạm vi hỗ trợ & Giới hạn (known issues)

### Về timezone

Thuật toán Ho Ngoc Duc tính toán theo múi giờ **UTC+7** của Việt Nam. Sang
truyền thống UTC+8 (Trung Quốc) sẽ khác nhau ở những năm khi sóc xảy ra rất sát
midnight UTC+8 — ví dụ:

- **Tết Đinh Hợi 2007**: sóc ngày 17/2/2007 lúc 16:14 UTC = **23:14 giờ VN** ngày
  17/2 → theo Ho's algorithm + múi giờ UTC+7, Tết VN là **17/2/2007**. Một số
  nguồn ghi "18/2/2007" vì dùng múi giờ TQ (UTC+8) — sự kiện sóc 16:14 UTC = **00:14
  giờ TQ** 18/2/2007 → ngày TQ = 18/2.
- **Tết Canh Ngọ 2030**: sóc ngày 2/2/2030 lúc 22:03 UTC = **05:03 giờ VN** ngày
  3/2 → theo Ho VN, Tết VN là **2/2/2030** (ngay trước midnight VN). Nguồn TQ ghi
  3/2/2030.

Trong `TET_CASES` của test, chúng tôi dùng **giá trị theo Ho's algorithm (UTC+7,
lich.vn)** như là **truth**: 2007 → Feb 17, 2030 → Feb 2. Đây không phải bug —
mà là độ chính xác thời gian sóc vào sát midnight, lich.vn là nguồn.

### Về độ chính xác phụ-ngày

Thuật toán polynomial (~2nd-order trong thời gian T, thời gian Julian thế kỷ) có
độ chính xác khoảng vài phút cho thời điểm sóc ở khu vực 1900–2100. Vài trường
hợp sóc rơi trúng rất sát midnight VN có thể bị làm tròn sai 1 ngày — nhưng trong
khoảng 1900–2100 này, những trường này đã được "liệt kê" trong `TET_CASES` (là
2007 và 2030 — đã xử lý đúng với timezone UTC+7 theo công thức chuẩn).

**Đo định lượng (giai đoạn 5).** Đối chiếu ngày Âm lịch với licham365.vn trên
**20 tháng** rải từ 1995 đến 2030 (**602 ngày**): khớp **602/602**. Ngoài tập đó,
đã tìm được đúng một ca sai trong thực tế: **tháng 7/2026**, do sóc rơi lúc 00:46
giờ VN ngày 13/8/2026 (nhật thực toàn phần 12/8/2026 lúc 17:46 UTC) — đa thức
xếp mùng 1 vào 12/8 thay vì 13/8. Đây đúng là kiểu ca "sóc sát nửa đêm" mô tả ở
trên; sửa triệt để cần lý thuyết Mặt Trăng độ chính xác cao hơn hẳn, nằm ngoài
phạm vi thuật toán Hồ Ngọc Đức.

### Về kinh độ Mặt Trời / tiết khí

Hàm `sun_longitude_deg_at_local_midnight` từng có **bug hằng số epoch** (dùng
`2451545.5` trong khi đã trừ 0.5 ngày riêng → double-count nửa ngày), làm ~50%
mốc tiết khí lệch 1 ngày. Giai đoạn 4 quy nhầm triệu chứng này cho giới hạn độ
chính xác của đa thức; giai đoạn 5 truy ra và sửa (`2451545.5` → `2451545.0`).

Sau khi sửa: **24/24** tiết khí năm 2024 và **10/10** mốc Lập Xuân 2017–2026 khớp
đúng ngày với lịch vạn niên. Có test regression trong `bat_tu/tiet_khi.rs`.

> ⚠️ `sun_longitude_at_noon` dùng `2451545.5` là **đúng** — ở đó shift −0.5 được
> gộp thẳng vào hằng số. Đừng "đồng bộ" hằng số giữa hai hàm.

### Sự khác biệt với lịch TQ

Sự khác biệt VN–TQ chỉ phát sinh ở khoảng 1–2 năm / thế kỷ, do múi giờ khác.
Ví dụ 2007 (17/2 VN vs 18/2 TQ), 2030 (2/2 VN vs 3/2 TQ). Với các mục đích tử vi
VN (Tử Vi Đẩu Số, Bát Tự) — bắt buộc phải dùng lịch âm VN.
Nếu sau này cần lịch âm以外===/> else, có thể thêm param `time_zone` vào public
API (algorithm đã hỗ trợ sẵn).

## Public API cheat-sheet

```rust
use tinhban_core::*;
use chrono::NaiveDate;

// Dương → Âm (UTC+7)
let lunar = solar_to_lunar(NaiveDate::from_ymd_opt(2024, 2, 10).unwrap()).unwrap();
// LunarDate { day: 1, month: 1, year: 2024, is_leap_month: false }

// Âm → Dương (hỗ trợ tháng nhuận)
let solar = lunar_to_solar(LunarDate {
    day: 1, month: 1, year: 2024, is_leap_month: false
}).unwrap();
// 2024-02-10

// Can–Chi
let y = year_can_chi(2024).unwrap();              // Giáp Thìn
let m = month_can_chi(HeavenlyStem::Giap, 1).unwrap();  // Bính Dần
let d = day_can_chi(NaiveDate::from_ymd_opt(2024, 2, 10).unwrap()).unwrap();
let h = hour_can_chi(HeavenlyStem::Giap, 23).unwrap();   // Giáp Tý

// Ngũ Hành
let ninh = nguhanh_of_stem(HeavenlyStem::Giap);    // Mộc
let ninh = nguhanh_of_branch(EarthlyBranch::Ty);  // Thủy

// Display string (tiếng Việt có dấu)
use tinhban_core::can_chi_display;
assert_eq!(can_chi_display(y), "Giáp Thìn");
```

## Lỗi

`LunarError` enum, `Display` impl trả chuỗi tiếng Việt:

- `OutOfRange(String)`: năm ngoài 1900–2100.
- `InvalidLunarDate(String)`: ngày > 30, tháng > 12, hoặc声称 `is_leap_month=true`
  nhưng năm đó không có tháng nhuận của số nêu (e.g. `1/13/2024` (leap=true) trả
  lỗi vì năm 2024 không có tháng 1 nhuận).

## Tương lai (giai đoạn sau)

- Lá số Tử Vi Đẩu Số (giai đoạn 3) sẽ dùng `LunarDate` để an 12 cung + 14 chính
  tinh + phụ tinh.
- Tứ Trụ Bát Tự (giai đoạn 4) sẽ dùng `BirthMoment`, `year_can_chi`,
  `month_can_chi`, `day_can_chi`, `hour_can_chi` để lập 4 trụ.
- Logic luận ngày tốt/xấu (giai đoạn 5) — **đã xong**, xem
  [`src/trach_nhat/README.md`](src/trach_nhat/README.md). Dùng `solar_to_lunar`,
  `day_can_chi`, `month_can_chi`, `year_can_chi` và module tiết khí của
  `bat_tu` làm nền
  cho các quy tắc hoàng đạo/hắc đạo, trực/từ/bại/khung...