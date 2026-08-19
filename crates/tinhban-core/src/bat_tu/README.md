# tinhban-core / bat_tu — Engine Bát Tự (Tứ Trụ)

Engine lập lá số Bát Tự (Four Pillars / Tứ Trụ) — 8 chữ Can-Chi cho
năm/tháng/ngày/giờ sinh + thống kê Ngũ Hành + Thập Thần + Tàng Can.

## Phạm vi đã implement

1. **Tứ Trụ** (4 cặp Can-Chi):
   - **Trụ Năm**: Can-Chi của năm Bát Tự (tính theo tiết khí **Lập Xuân**
     315°, không phải Tết Nguyên Đán). Sinh trước Lập Xuân (~4-5/2 Dương)
     → trụ Năm thuộc năm Can Chi cũ. Logic tiết khí tái sử dụng công thức
     thiên văn Hồ Ngọc Đức từ giai đoạn 2 (`sun_longitude_deg_at_local_midnight`
     trong `astronomy.rs`), tìm JD khi kinh độ Mặt Trời vượt qua target_deg.
   - **Trụ Tháng**: Can-Chi của tháng Bát Tự (theo 12 **tiết** — Lập Xuân,
     Kinh Trập, Thanh Minh, ..., Tiểu Hàn — không phải tháng âm lịch Tử Vi).
     Mỗi tháng BT ứng với 1 Địa Chi cố định: tháng 1 = Dần, tháng 2 = Mão,
     ..., tháng 11 = Tý, tháng 12 = Sửu. Can của tháng tính bằng quy tắc
     "Ngũ Hổ Độn" (tái sử dụng `month_can_chi` từ giai đoạn 2).
   - **Trụ Ngày**: Can-Chi của ngày sinh (Julian Day công thức chuẩn
     `(jd+9)%10`, `(jd+1)%12` — tái sử dụng `day_can_chi` từ giai đoạn 2).
   - **Trụ Giờ**: Can-Chi của giờ sinh (12 giờ Địa Chi, giờ Tý = 23h-1h,
     theo "Ngũ Thử Độn Thời" — tái sử dụng `hour_can_chi` từ giai đoạn 2).
2. **Thống kê Ngũ Hành**: đếm số lần mỗi hành (Kim/Mộc/Thủy/Hỏa/Thổ)
   xuất hiện trong 8 chữ (4 Can + 4 Chi). Dùng `nguhanh_of_stem` /
   `nguhanh_of_branch` từ giai đoạn 2. Output `NguHanhCount` với tổng = 8.
3. **Thập Thần** (Ten Gods): tính cho 3 Can của trụ Năm/Tháng/Giờ so với
   Nhật Chủ (= Can của trụ Ngày) theo quy tắc Sinh/Khắc Ngũ Hành + Âm Dương.
   Trụ Ngày có `ten_god = None` (đó chính là Nhật Chủ).
4. **Tàng Can** (Hidden Stems): mỗi Địa Chi có 1-3 Can ẩn theo bảng cố định.
   Mỗi Tàng Can kèm Thập Thần tương ứng so với Nhật Chủ. Bảng đối chiếu
   chuẩn Bát Tự Bắc Tông (xem `hidden_stems.rs` cho bảng đầy đủ).

## Cố ý CHƯA implement

- **Vượng / Suy Nhật Chủ** — luận ngày được/không được mùa. Nhiều trường
  phái tranh cãi, cần luận giải sâu, để sau.
- **Dụng Thần / Kỵ Thần** — chọn hành cần bổ sung/tránh. Phụ thuộc luận
  Vượng-Suy, để sau.
- **Đại Vận** (Luck Pillars — vận trình 10 năm) — tính dựa theo giới
  tính × Can năm × chiều thuận/nghịch + khoảng cách tới tiết khí gần nhất.
- **Lưu Niên** (vận từng năm).
- **UI hiển thị lá số Bát Tự** — giai đoạn 6.

## Cách xử lý tiết khí cho trụ Năm/Tháng

Bát Tự khác Tử Vi ở chỗ năm/tháng đổi theo **tiết khí** (節氣), không theo
Tết Nguyên Đán hoặc tháng âm lịch:

- **Năm BT** bắt đầu tại **Lập Xuân** (315° kinh độ Mặt Trời, ~4-5/2 Dương).
  Sinh 29/01/1990 (trước Lập Xuân 1990) → trụ Năm = Kỷ Tỵ (1989), không phải
  Canh Ngọ (1990) dù âm lịch đã sang năm Canh Ngọ.
- **Tháng BT** đổi tại 12 tiết (cách nhau 30°): Lập Xuân → Kinh Trập → Thanh
  Minh → ... → Tiểu Hàn. Sinh 24/10/1991 → tháng BT = Tuất (month 9), không
  phải tháng 9 âm lịch.

Module `tiet_khi.rs` tìm JD của mỗi tiết bằng cách walk-forward từng ngày
trong năm, tính kinh độ Mặt Trời tại local-midnight VN (dùng `sun_longitude_
deg_at_local_midnight` từ `astronomy.rs` — công thức Ho's NEW với hiệu chỉnh
nutation), phát hiện ngày "transition" (kinh độ vượt qua target_deg). Calendar
day containing transition = tiết khí day.

### ~~Giới hạn độ chính xác tiết khí (±1 ngày)~~ — ĐÃ RÚT LẠI

> ⚠️ **Mục này từng khẳng định sai và đã được rút lại ở giai đoạn 5.**
>
> Nội dung cũ: *"Công thức polynomial của Hồ Ngọc Đức có độ chính xác ~±0.5° …
> 4/6 năm Lập Xuân thử nghiệm lệch 1 ngày so với HK Observatory. Khuyến nghị:
> ngày sinh cách biên tiết ≥ 5 ngày."*
>
> Triệu chứng quan sát được là **thật**, nhưng nguyên nhân quy sai. Đó không
> phải giới hạn của thuật toán mà là **bug hằng số epoch** trong
> `sun_longitude_deg_at_local_midnight` (xem mục audit ở cuối file). Sau khi sửa,
> đối chiếu Đài Thiên văn Hồng Kông: **192/192 mốc khớp** (trước: 97/192).
>
> **Khuyến nghị "cách biên ≥ 5 ngày" không còn hiệu lực** — ranh giới tháng/năm
> Bát Tự nay đáng tin cả với ngày sinh rơi đúng mốc tiết khí, và đã có 11 test
> case biên chứng minh điều đó.

## Convention hour boundary

Trụ Ngày dùng Julian Day của `birth.solar_date` (calendar day tại UTC+7). Sinh
ở 23:30 hôm nay theo convention này vẫn thuộc **nay** (không phải "giờ Tý của
ngày mai"). Một số trường phái Bát Tự cổ truyền dùng convention "giờ Tý (23:00)
đánh dấu ngày mới" → lịch khác cách 1 ngày; module này theo convention lasotuvi
/ hiện đại cho nhất quán với phase 2 `hour_can_chi`.

## Nguồn đối chiếu test

**Chính (7 case gốc)**: Python prototype dùng **cùng** Ho's algorithm (tái sử
dụng `tiet_khi_proto.py` từ giai đoạn 2 + `gen_bat_tu_ref.py`), port vào
`tests/bat_tu_ref_data.rs`. Convention: tiet_khi JD = calendar day **chứa**
transition (= jd - 1 so với JD nơi L_first >= target).

> ⚠️ **Điểm yếu đã lộ ra ở giai đoạn 5**: reference này **tự sinh từ chính thuật
> toán đang được kiểm** — nên nó không thể phát hiện lỗi nằm trong thuật toán đó.
> Bug #7 (epoch sai) sống sót qua toàn bộ test giai đoạn 4 đúng vì lý do này.
> Bộ case biên bổ sung (dưới đây) lấy giá trị kỳ vọng từ **nguồn ngoài độc lập**
> để không lặp lại sai lầm.

**Bổ sung (11 case biên, giai đoạn 5)**: mốc tiết khí lấy từ **Đài Thiên văn
Hồng Kông** (đã quy giờ HK→VN), trụ Ngày đối chiếu thêm với licham365.vn —
`tests/bat_tu_boundary_ref_data.rs`. Xem mục audit ở cuối file.

### 7 lá số mẫu

| Case  | Dương lịch  | Giờ | BT Year | BT Month | Year Pillar | Month Pillar |
|-------|-------------|-----|---------|----------|-------------|--------------|
| case1 | 24/10/1991  | 7   | 1991    | 9 (Tuất) | Tân Mùi     | Mậu Tuất     |
| case2 | 17/02/2026  | 12  | 2026    | 1 (Dần)  | Bính Ngọ    | Canh Dần     |
| case3 | 10/02/2024  | 0   | 2024    | 1 (Dần)  | Giáp Thìn   | Bính Dần     |
| case4 | 29/01/1990  | 9   | 1989    | 12 (Sửu)| Kỷ Tỵ       | Đinh Sửu     |
| case5 | 05/05/2000  | 15  | 2000    | 4 (Tỵ)   | Canh Thìn   | Tân Tỵ       |
| case6 | 06/02/1991  | 14  | 1991    | 1 (Dần)  | Tân Mùi     | Canh Dần     |
| case7 | 01/02/1990  | 8   | 1989    | 12 (Sửu)| Kỷ Tỵ       | Đinh Sửu     |

Cases 4, 6, 7 là edge cases gần Lập Xuân (trước/sau 1-5 ngày). Case 5 ở
ranh tiết Lập Hạ (5/5/2000 = ngày tiết Lập Hạ).

> ⚠️ Audit giai đoạn 5 cho thấy 7 case này **không phủ được vùng rủi ro của Bug
> #7**, và case5 chỉ đúng **nhờ may** (mốc Lập Hạ 2000 tình cờ không bị bug đẩy
> lệch). Bảng khoảng cách tới mốc tiết của từng case ở mục audit cuối file.

### Kết quả

```
running 8 tests
test all_reference_bt_cases_match_can_chi_indices ... ok
test day_pillar_ten_god_is_none ... ok
test hidden_stems_attached_to_each_pillar ... ok
test lap_xuan_edge_case_one_day_post_returns_next_year ... ok
test lap_xuan_edge_case_three_days_pre_returns_previous_bt_year ... ok
test lap_xuan_pre_birth_year_returns_previous_bt_year ... ok
test nhat_chu_is_day_pillar_stem ... ok
test non_day_pillars_have_ten_god ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Tất cả 56 Tứ Trụ Can/Chi (7 cases × 4 pillars × 2 = 56) khớp reference.

## Bảng Tàng Can (Hidden Stems) — test riêng

Bảng đối chiếu chuẩn Bát Tự Bắc Tông trong `hidden_stems.rs` có test riêng
so sánh từng Chi với reference:

| Chi    | Tàng Can (bản → trung → dư)  |
|--------|-------------------------------|
| Tý     | Quý                           |
| Sửu    | Kỷ, Quý, Tân                  |
| Dần    | Giáp, Bính, Mậu               |
| Mão    | Ất                            |
| Thìn   | Mậu, Ất, Quý                  |
| Tỵ     | Bính, Mậu, Canh               |
| Ngọ    | Đinh, Kỷ                      |
| Mùi    | Kỷ, Đinh, Ất                  |
| Thân   | Canh, Nhâm, Mậu               |
| Dậu    | Tân                           |
| Tuất   | Mậu, Tân, Đinh               |
| Hợi    | Nhâm, Giáp                   |

## Thập Thần (Ten Gods) — test riêng

Bảng quy tắc Sinh/Khắc/Âm-Dương → TenGod được test bằng 2 bộ:
1. **Nhật chủ Giáp (Dương Mộc)** vs 10 Can: kiểm tra đủ 10 Thập Thần.
2. **Nhật chủ Đinh (Âm Hỏa)** vs 10 Can: kiểm tra đủ 10 Thập Thần.

Bảng quy tắc:

| Quan hệ ngũ hành       | Cùng âm dương | Khác âm dương |
|------------------------|---------------|---------------|
| Cùng hành              | Tỷ Kiên       | Kiếp Tài      |
| Nhật chủ sinh can      | Thực Thần     | Thương Quan   |
| Nhật chủ khắc can      | Thiên Tài     | Chính Tài     |
| Can khắc Nhật chủ      | Thất Sát       | Chính Quan    |
| Can sinh Nhật chủ      | Thiên Ấn       | Chính Ấn      |

## Public API

```rust
use tinhban_core::{lap_bat_tu, BirthMoment, Gender};
use chrono::NaiveDate;

let birth = BirthMoment {
    solar_date: NaiveDate::from_ymd_opt(1991, 10, 24).unwrap(),
    hour: 7,
    minute: 0,
};
let chart = lap_bat_tu(birth, Gender::Nam).unwrap();

// Tứ Trụ
println!("Năm: {} {}", chart.year_pillar.can_chi.stem.name_vn(),
         chart.year_pillar.can_chi.branch.name_vn());
println!("Tháng: {} {}", chart.month_pillar.can_chi.stem.name_vn(),
         chart.month_pillar.can_chi.branch.name_vn());
println!("Ngày: {} {}", chart.day_pillar.can_chi.stem.name_vn(),
         chart.day_pillar.can_chi.branch.name_vn());
println!("Giờ: {} {}", chart.hour_pillar.can_chi.stem.name_vn(),
         chart.hour_pillar.can_chi.branch.name_vn());

// Nhật Chủ
println!("Nhật Chủ: {}", chart.nhat_chu().name_vn());

// Thập Thần của trụ Năm
if let Some(tg) = chart.year_pillar.ten_god {
    println!("Thập Thần trụ Năm: {}", tg.name_vn());
}

// Thống kê Ngũ Hành
println!("Kim={}, Mộc={}, Thủy={}, Hỏa={}, Thổ={}",
    chart.nguhanh_count.kim, chart.nguhanh_count.moc,
    chart.nguhanh_count.thuy, chart.nguhanh_count.hoa,
    chart.nguhanh_count.tho);

// Tàng Can của trụ Năm
for (stem, tg) in &chart.year_pillar.hidden_stems {
    println!("Tàng Can: {} ({})", stem.name_vn(), tg.name_vn());
}
```

## Bản đồ module `bat_tu/`

```
bat_tu/
├── mod.rs           entry point `lap_bat_tu`, helper `check_range`
├── types.rs          `BatTuChart`, `Pillar`, `NguHanhCount`, `TenGod`, `BatTuError`
├── tiet_khi.rs       `TIET_KHI_TABLE`, `find_tiet_khi_jd`, `lap_xuan_jd`, `tiet_khi_jds_of_bt_year`
├── hidden_stems.rs   `hidden_stems`, `hidden_stems_tuple` + bảng 12 Chi
└── thap_than.rs      `TenGod` enum, `ten_god_of` + bảng Sinh/Khắc
```


---

# Ghi chú hồi tố: audit Bug #7 (epoch sai) — ảnh hưởng ngược tới giai đoạn 4

> Bug được **phát hiện ở giai đoạn 5** (Trạch Nhật) nhưng **có nguồn gốc từ
> giai đoạn 4**: hàm bị lỗi được viết ra và dùng lần đầu chính ở
> `bat_tu/tiet_khi.rs`. Mục này ghi lại toàn bộ kết quả audit để có audit trail
> rõ ràng.

## Bug là gì

`sun_longitude_deg_at_local_midnight` trong `astronomy.rs` tính
`real_jd = jdn − 0.5 − tz/24` (đã trừ nửa ngày), rồi lại dùng epoch `2451545.5`
— vốn **đã gộp sẵn** nửa ngày đó. Kết quả: double-count 0.5 ngày → kinh độ Mặt
Trời thấp hơn thực tế ~0.49°.

README này trước đây ghi triệu chứng ("6 năm Lập Xuân thử nghiệm có 4/6 lệch 1
ngày so với HK Observatory") và quy cho **giới hạn độ chính xác của đa thức Hồ
Ngọc Đức**. Kết luận đó **sai**: đây là bug hằng số, sửa được.

Sửa: `2451545.5` → `2451545.0` (một hằng số, một dòng).

## Chiều sai và phạm vi

Kinh độ bị tính thấp → mốc tiết khí bị đẩy **TRỄ**. Đo trên **toàn bộ 1900–2100,
12 tiết × 201 năm = 2412 mốc**:

| Lệch (mã cũ − mã đúng) | Số mốc | Tỉ lệ |
|---|---|---|
| +1 ngày (trễ) | 1205 | 50.0% |
| 0 ngày | 1207 | 50.0% |
| −1 ngày (sớm) | **0** | 0% |

Sai **một chiều tuyệt đối** — không bao giờ sớm. Riêng Lập Xuân: **100/201 năm**
bị đẩy trễ 1 ngày. Mỗi năm có 3–9 trong 12 mốc bị lệch.

Hệ quả: ngày sinh rơi **đúng vào ngày giao tiết thật** bị xếp nhầm vào kỳ
TRƯỚC đó. Vùng rủi ro rộng đúng **1 ngày** cho mỗi mốc bị lệch.

## Lá số Bát Tự bị sai bao nhiêu

Mô phỏng lại đúng logic `lap_bat_tu` với cả hai hằng số, quét từng ngày
**1960–2030 (25 933 ngày)**:

| Hạng mục | Số ngày | Tỉ lệ |
|---|---|---|
| Sai **trụ Năm** | 35 | 0.135% |
| Sai **trụ Tháng** (trụ Năm vẫn đúng) | 389 | 1.50% |
| **Tổng ngày cho lá số sai** | **424** | **1.635%** |

- **Vùng rủi ro trụ Năm**: toàn bộ 35 ngày đều rơi vào **3/2 hoặc 4/2** — tức
  người sinh đúng ngày Lập Xuân, trong 35 năm bị ảnh hưởng:
  1962, 1963, 1966, 1967, 1970, 1971, 1975, 1976, 1979, 1980, 1983, 1984, 1987,
  1988, 1991, 1992, 1995, 1996, 1999, 2000, 2003, 2004, 2007, 2008, 2009, 2012,
  2013, 2016, 2017, 2020, 2021, 2024, 2025, 2028, 2029.
  Sai kiểu này nghiêm trọng nhất vì **lệch hẳn sang năm Can Chi liền trước**
  (ví dụ 4/2/2024 bị tính Quý Mão thay vì Giáp Thìn).
- **Vùng rủi ro trụ Tháng**: rải đều ~50 ngày/thập niên, luôn là ngày giao tiết
  của một trong 12 tiết → lệch sang **tháng Can Chi liền trước**.

## Kiểm chứng bản sửa với nguồn ngoài

Nguồn: **Đài Thiên văn Hồng Kông** (Hong Kong Observatory), bảng
`https://www.hko.gov.hk/en/gts/time/calendar/text/files/T{year}e.txt` — cơ quan
khí tượng/thiên văn chính thức của HK.

HKO công bố theo giờ HK (**UTC+8**), ta cần giờ VN (**UTC+7**), nên mốc rơi trong
khoảng **00:00–00:59 giờ HK** phải lùi 1 ngày khi quy về VN. Trong mẫu dưới đây
có đúng 8 mốc như vậy (rơi 00:03–00:51 giờ HK).

Đối chiếu **16 năm × 12 tiết = 192 mốc** (1960, 1970, 1980, 1985, 1990, 1991,
1995, 2000, 2005, 2010, 2015, 2020, 2024, 2025, 2026, 2030), đã quy về giờ VN:

| | Khớp HKO |
|---|---|
| **Sau** khi sửa epoch | **192/192 (100%)** |
| Trước khi sửa | 97/192 (50.5%) |

## Đánh giá lại 7 case Bát Tự của giai đoạn 4

Câu hỏi phải trả lời: chúng "vẫn xanh" vì **miễn nhiễm** hay vì **may**?

| Case | Ngày sinh | Mốc tiết gần nhất | Khoảng cách | Bị ảnh hưởng? |
|---|---|---|---|---|
| case1 | 1991-10-24 | Hàn Lộ 9/10/1991 | +15 ngày | không |
| case2 | 2026-02-17 | Lập Xuân 4/2/2026 | +13 ngày | không |
| case3 | 2024-02-10 | Lập Xuân 4/2/2024 | +6 ngày | không |
| case4 | 1990-01-29 | Lập Xuân 4/2/1990 | −6 ngày | không |
| **case5** | **2000-05-05** | **Lập Hạ 5/5/2000** | **0 ngày** | **không — nhưng chỉ nhờ may** |
| case6 | 1991-02-06 | Lập Xuân 4/2/1991 | +2 ngày | không |
| case7 | 1990-02-01 | Lập Xuân 4/2/1990 | −3 ngày | không |

**Kết luận: 0/7 case bị ảnh hưởng — nhưng bộ test cũ KHÔNG hề miễn nhiễm theo
thiết kế.**

`case5` nằm **đúng trên mốc Lập Hạ 2000** — vị trí rủi ro cao nhất có thể. Nó
đúng chỉ vì Lập Hạ 2000 tình cờ thuộc 50% số mốc mà bug không đẩy lệch. Nếu ngày
sinh mẫu đó rơi vào một mốc thuộc nửa còn lại, giai đoạn 4 đã đỏ ngay từ đầu.

Ngoài ra `case3` và `case6` có mốc Lập Xuân bên cạnh **thực sự bị bug đẩy lệch**
(4/2 → 5/2), chỉ thoát vì ngày sinh cách mốc 6 và 2 ngày — xa hơn mức lệch 1 ngày.

Kết luận này được khoá bằng test
`bay_case_giai_doan_4_ngoai_vung_rui_ro_nhung_case5_chi_thoat_nho_may` trong
`tests/bug7_epoch_audit.rs`, để nó là **sự thật kiểm chứng được** chứ không phải
lời khẳng định trong tài liệu.

## Bộ case biên bổ sung

Vì bộ cũ không phủ vùng rủi ro, đã thêm **11 case nhắm thẳng vào biên tiết khí**
(`tests/bat_tu_boundary_ref_data.rs`), giá trị kỳ vọng suy từ HKO:

- **5 case sinh đúng ngày Lập Xuân** (ranh giới trụ Năm — quan trọng nhất):
  1980, 1991, 2000, 2024, 2025;
- **3 case đối chứng** sinh trước Lập Xuân 1 ngày — bảo đảm bản sửa không "chữa
  quá tay" đẩy ranh giới lệch sang chiều ngược lại;
- **3 case sinh đúng ngày giao tiết khác** (ranh giới trụ Tháng): Tiểu Hàn 2010,
  Thanh Minh 2020, Bạch Lộ 1985.

**8/11 case này bị mã trước khi sửa tính SAI** (5 sai trụ Năm, 3 sai trụ Tháng):

| Case | Mã cũ | Đúng |
|---|---|---|
| 1980-02-04 | Năm Kỷ Mùi, Tháng Đinh Sửu | Năm **Canh Thân**, Tháng **Mậu Dần** |
| 1991-02-04 | Năm Canh Ngọ, Tháng Kỷ Sửu | Năm **Tân Mùi**, Tháng **Canh Dần** |
| 2000-02-04 | Năm Kỷ Mão, Tháng Đinh Sửu | Năm **Canh Thìn**, Tháng **Mậu Dần** |
| 2024-02-04 | Năm Quý Mão, Tháng Ất Sửu | Năm **Giáp Thìn**, Tháng **Bính Dần** |
| 2025-02-03 | Năm Giáp Thìn, Tháng Đinh Sửu | Năm **Ất Tỵ**, Tháng **Mậu Dần** |
| 2010-01-05 | Tháng Bính Tý | Tháng **Đinh Sửu** |
| 2020-04-04 | Tháng Kỷ Mão | Tháng **Canh Thìn** |
| 1985-09-07 | Tháng Giáp Thân | Tháng **Ất Dậu** |

Trụ Ngày của cả 11 case còn được đối chiếu độc lập với licham365.vn: **11/11
khớp**.

## Tử Vi Đẩu Số (giai đoạn 3) KHÔNG bị ảnh hưởng

`astronomy.rs` có **hai** hàm kinh độ Mặt Trời, đi hai nhánh tách biệt:

```text
sun_longitude_deg_at_local_midnight   ← BỊ LỖI
  └── bat_tu::tiet_khi::*
        ├── lap_xuan_jd             → lap_bat_tu (TRỤ NĂM)
        ├── tiet_khi_jds_of_bt_year → lap_bat_tu (TRỤ THÁNG)
        └── tiet_month_branch_index → trach_nhat::truc (12 Trực, giai đoạn 5)

sun_longitude_at_noon                 ← ĐÚNG (epoch đã gộp sẵn −0.5)
  └── get_sun_longitude
        └── solar_to_lunar / lunar_to_solar → Tử Vi, ngày Âm lịch
```

Tử Vi chỉ nhận đầu vào là **ngày Âm lịch + giờ sinh**, đi qua `solar_to_lunar` —
nhánh có epoch đúng. Module `tuvi/` không import `astronomy` hay `tiet_khi` dưới
bất kỳ hình thức nào.

Điều này được khoá bằng **hai** test trong `tests/bug7_epoch_audit.rs`:

- `tuvi_khong_phu_thuoc_astronomy_hay_tiet_khi` — kiểm ở mức **mã nguồn** (quét
  cả 8 file của `tuvi/`, cấm nhắc tới `astronomy` / `tiet_khi` / `lap_xuan` /
  `sun_longitude`). Nếu sau này ai thêm phụ thuộc tiết khí vào Tử Vi, test đỏ và
  buộc audit lại thay vì để kết luận này âm thầm hết đúng.
- `tuvi_khong_doi_cung_menh_qua_moc_lap_xuan` — kiểm ở mức **hành vi**: 3/2 và
  4/2/2024 nằm hai bên mốc Lập Xuân; Bát Tự đổi hẳn trụ Năm (Quý Mão → Giáp
  Thìn) trong khi cung Mệnh Tử Vi giữ nguyên.

## Rủi ro thực tế

**Bằng 0.** Bảng hồ sơ người được xem + lá số đã lập (`tinhban-db`) chưa được
implement — dự kiến giai đoạn 6. Không có lá số nào từng được tạo và lưu bằng mã
lỗi, nên không cần migration dữ liệu hay thông báo tính lại.

Ghi chép này tồn tại cho mục đích minh bạch / audit trail, không phải để xử lý
sự cố.

## Cập nhật ở giai đoạn 5: sửa bug epoch của tiết khí

`sun_longitude_deg_at_local_midnight` (dùng cho mọi mốc tiết khí ở đây) từng có
bug hằng số epoch làm ~50% mốc tiết khí lệch đúng 1 ngày — chính là hiện tượng
"±1 ngày, 4/6 mốc Lập Xuân lệch" mà README này từng quy cho giới hạn độ chính
xác của đa thức Hồ Ngọc Đức. Thực ra là bug hằng số, đã sửa
(`2451545.5` → `2451545.0`).

Sau khi sửa, đối chiếu lịch vạn niên: **24/24** tiết khí năm 2024 và **10/10**
mốc Lập Xuân 2017–2026 khớp đúng ngày. Nhờ vậy ranh giới **tháng/năm Bát Tự**
nay đáng tin cả với ca sinh sát mốc tiết — khuyến nghị cũ "cách biên tiết ít nhất
5 ngày" không còn cần thiết.

8 test đối chiếu Bát Tự vẫn xanh sau khi sửa: các ca mẫu đều đủ xa biên nên kết
luận không đổi.

Đồng thời sửa 2 tên sai trong `TIET_KHI_TABLE`: 75° là **Mang Chủng** (không phải
Tiểu Mãn) và 105° là **Tiểu Thử** (không phải Mang Chủng). Kinh độ vốn đúng nên
tháng Bát Tự không bị ảnh hưởng — chỉ sai nhãn hiển thị.

Bảng đầy đủ 24 tiết khí (12 tiết 節 + 12 trung khí 氣) nay có ở `TIET_KHI_24`,
kèm `tiet_month_index` / `tiet_month_branch_index` / `current_tiet_khi` — các hàm
này chỉ tốn **1** lần tính kinh độ Mặt Trời (thay vì quét cả năm), dùng được cho
từng ngày mà không lo hiệu năng. Giai đoạn 5 (Trạch Nhật) dùng chúng cho 12 Trực.
