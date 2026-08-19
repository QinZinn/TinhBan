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

### Giới hạn độ chính xác tiết khí (±1 ngày)

Công thức polynomial của Hồ Ngọc Đức có độ chính xác ~±0.5° cho kinh độ Mặt
Trời. Vì Sun tiến ~0.985°/ngày, ±0.5° ≈ ±0.5 ngày. Trong thực tế: 4/6 năm
Lập Xuân thử nghiệm lệch 1 ngày so với nguồn chính thức (HK Observatory).
**Khuyến nghị**: ngày sinh cách biên tiết ≥ 5 ngày để BT year/month được xác
định unambiguously. Lá số ở giữa tháng BT không bị ảnh hưởng.

## Convention hour boundary

Trụ Ngày dùng Julian Day của `birth.solar_date` (calendar day tại UTC+7). Sinh
ở 23:30 hôm nay theo convention này vẫn thuộc **nay** (không phải "giờ Tý của
ngày mai"). Một số trường phái Bát Tự cổ truyền dùng convention "giờ Tý (23:00)
đánh dấu ngày mới" → lịch khác cách 1 ngày; module này theo convention lasotuvi
/ hiện đại cho nhất quán với phase 2 `hour_can_chi`.

## Nguồn đối chiếu test

**Chính**: Python prototype dùng cùng Ho's algorithm (tái sử dụng `tiet_khi_
proto.py` từ giai đoạn 2 + `gen_bat_tu_ref.py`). Reference data sinh bằng
script Python rồi port vào `tests/bat_tu_ref_data.rs`. Convention: tiet_khi
JD = calendar day **chứa** transition (= jd - 1 so với JD nơi L_first >= target).

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
ranh tiết Lập Hạ (5/5/2000 = ngày tiết Lập Hạ theo Ho's algorithm).

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