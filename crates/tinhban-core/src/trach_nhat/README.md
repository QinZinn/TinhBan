# tinhban-core / trach_nhat — Engine Trạch Nhật (xem ngày tốt/xấu)

Cho một ngày Dương lịch, trả về đánh giá tốt/xấu theo các yếu tố truyền thống:
ngày Hoàng Đạo/Hắc Đạo, giờ Hoàng Đạo trong ngày, 12 Trực, và các ngày kiêng kỵ
phổ biến. **Thuần offline** — không mạng, không DB.

Phần diễn giải văn bản lấy từ licham365.vn nằm ở tầng `tinhban-api`
(`src/scrape/`) và chỉ *bổ sung* lên kết quả này.

---

## 1. Bảng tra nhanh: mỗi thành phần neo theo cái gì

Đây là bảng quan trọng nhất của module. Ba thành phần dùng **ba loại "tháng"
khác nhau**, và dùng nhầm là lỗi phổ biến nhất khi tự implement Trạch Nhật.

| Thành phần | Neo theo | Công thức |
|---|---|---|
| **Giờ** Hoàng Đạo/Hắc Đạo | Chi của **ngày** | `khởi = (2·chi_ngày + 8) mod 12` |
| **Ngày** Hoàng Đạo/Hắc Đạo | Chi của tháng **Âm lịch** | `khởi = (2·chi_tháng_âm + 8) mod 12` |
| **12 Trực** | Chi của tháng **tiết khí** | `trực = (chi_ngày − chi_tháng_tiết) mod 12` |
| Tam Nương / Nguyệt Kỵ | Ngày **Âm lịch** | tra danh sách cố định |
| Sát Chủ | Chi ngày × tháng **Âm lịch** | tra bảng 12 dòng |

> **Đừng "thống nhất" tháng giữa các dòng.** Ngày Hoàng Đạo dùng tháng Âm lịch
> còn 12 Trực dùng tháng tiết khí — cả hai đều đã được đối chiếu số liệu thật
> (mục 5). Mốc tiết khí không trùng mùng 1 Âm lịch nên hai loại tháng lệch nhau
> vài ngày mỗi tháng.

---

## 2. Vòng 12 Thần (Hoàng Đạo / Hắc Đạo)

Thứ tự vòng **bất biến**, chỉ điểm khởi thay đổi:

```
0 Thanh Long · 1 Minh Đường · 2 Thiên Hình · 3 Chu Tước ·
4 Kim Quỹ    · 5 Thiên Đức  · 6 Bạch Hổ    · 7 Ngọc Đường ·
8 Thiên Lao  · 9 Nguyên Vũ  · 10 Tư Mệnh   · 11 Câu Trận
```

- **Hoàng Đạo (tốt)**: index 0, 1, 4, 5, 7, 10
- **Hắc Đạo (xấu)**: index 2, 3, 6, 8, 9, 11

Điểm khởi (vị trí Thanh Long) theo "chi gốc", tái tạo đúng bảng truyền thống:

| Chi gốc | Thanh Long đóng tại |
|---|---|
| Tý, Ngọ | Thân |
| Sửu, Mùi | Tuất |
| Dần, Thân | Tý |
| Mão, Dậu | Dần |
| Thìn, Tuất | Thìn |
| Tỵ, Hợi | Ngọ |

Công thức `(2·b + 8) mod 12` khớp cả 6 dòng (có test).

Ví dụ kiểm chứng: ngày **Dần/Thân** → giờ Hoàng Đạo là Tý, Sửu, Thìn, Tỵ, Mùi,
Tuất — khớp lịch vạn niên.

---

## 3. 12 Trực (Kiến Trừ Thập Nhị Khách)

```
Kiến · Trừ · Mãn · Bình · Định · Chấp · Phá · Nguy · Thành · Thu · Khai · Bế
```

Neo: **Trực Kiến rơi vào ngày có Chi trùng Chi của tháng tiết khí**, nên
`trực = (chi_ngày − chi_tháng_tiết) mod 12`.

**"Trùng Trực" tự sinh ra, không cần xử lý riêng.** Truyền thống nói ngày giao
tiết thì Trực lặp lại một ngày. Qua mốc tiết, `chi_tháng` tăng 1 đồng thời
`chi_ngày` cũng tăng 1 → hiệu số giữ nguyên → Trực lặp. Có test cho ca này.

---

## 4. Kiêng kỵ

| Mục | Quy tắc |
|---|---|
| **Tam Nương** | Ngày Âm lịch 3, 7, 13, 18, 22, 27 |
| **Nguyệt Kỵ** | Ngày Âm lịch 5, 14, 23 ("mùng năm, mười bốn, hai ba") |
| **Sát Chủ** | Tra `BANG_SAT_CHU` theo tháng Âm lịch × Chi ngày |

Ba mục **độc lập nhau và độc lập với Hoàng Đạo/Hắc Đạo** — một ngày Hoàng Đạo
vẫn có thể là Tam Nương. Vì vậy API trả **danh sách**, cố ý không quy về một
điểm số duy nhất: người dùng tự quyết định mình quan tâm điều nào.

### Bảng Sát Chủ được suy ra từ dữ liệu, không chép từ sách

Các bản "bảng Sát Chủ" lưu hành trên mạng mâu thuẫn nhau, nên bảng trong
`kieng_ky.rs` được **đo trực tiếp**: với mỗi tháng Âm lịch, quét 12 ngày liên
tiếp (đủ trọn 12 Địa Chi, mỗi Chi đúng một lần) trên licham365.vn và ghi lại
ngày nào bị gắn nhãn "Ngày sát chủ" — lặp cho **2 năm Âm lịch độc lập (2024 và
2025)**, tổng **288 trang ngày**.

**Kết quả: hai năm trùng khớp 12/12 tháng** → xác nhận đây là bảng tra cố định
theo tháng Âm lịch, không phụ thuộc năm, không phụ thuộc Can của ngày.

Mỗi tháng có 2–3 Chi vì licham365 gắn nhãn theo **hợp** của nhiều dòng truyền
(Sát Chủ Dương, Sát Chủ Âm, …) chứ không theo một bảng đơn lẻ. Ai chỉ muốn kiêng
theo một dòng cụ thể sẽ thấy bảng này báo nhiều ngày hơn mong đợi — lựa chọn có
chủ đích để khớp nguồn đối chiếu.

---

## 5. Đối chiếu với nguồn ngoài

Nguồn đối chiếu: **licham365.vn** (không phải tự sinh từ code của dự án — khác
với `bat_tu_ref_data.rs` giai đoạn 4 vốn sinh từ prototype Python nội bộ).

### 5.1 Quy tắc nào đúng — đo trên 174 ngày

Trước khi chốt công thức, cả hai lựa chọn "tháng Âm lịch" và "tháng tiết khí"
đều được thử trên cùng tập dữ liệu:

| Thành phần | Dùng tháng **tiết khí** | Dùng tháng **Âm lịch** |
|---|---|---|
| 12 Trực | **172/174** ✅ | 149/174 ❌ |
| Ngày Hoàng Đạo/Hắc Đạo | 100/109 ❌ | **107/109** ✅ |

(Ngày Hoàng Đạo chỉ tính trên 109 ngày mà licham365 kết luận dứt khoát
tốt/xấu — xem 5.3.)

Con số này là lý do bảng ở mục 1 dùng hai loại tháng khác nhau.

### 5.2 Giờ Hoàng Đạo: 174/174

Khớp tuyệt đối, không ngoại lệ.

### 5.3 Bộ test đối chiếu: 52 ngày

`tests/trach_nhat_reference.rs` + `tests/trach_nhat_ref_data.rs`:

- 24 ngày rải đều **1995–2030**, nhiều mùa, gồm cả đầu và cuối tháng Âm lịch;
- 6 ngày **"trùng Trực"** tại mốc giao tiết khí;
- 6 ngày **tháng Âm lịch ≠ tháng tiết khí** (ca phân biệt quy tắc Trực);
- 10 ngày dính **Tam Nương / Nguyệt Kỵ / Sát Chủ**, gồm ngày dính 2 mục;
- 6 ngày **lệch đã biết**, giữ lại có chủ đích (xem dưới).

Test kiểm tra **hai chiều**: ngày không khai báo mà lệch → fail; ngày đã khai
báo lệch mà lại khớp → **cũng fail**, buộc phải xoá mục lỗi thời. Danh sách miễn
trừ không thể âm thầm mục ruỗng thành cái cớ bỏ qua lỗi.

Ngoài ra `Can Chi ngày` và `giờ Hoàng Đạo` là thuần số học nên **không được phép
lệch** — sai là `assert!` panic ngay, không qua danh sách miễn trừ.

### 5.4 Bốn nguyên nhân lệch đã hiểu rõ

| Nguyên nhân | Số ca | Bên nào đúng |
|---|---|---|
| Sóc rơi sát nửa đêm giờ VN → lệch 1 ngày Âm | 1 tháng | **licham365 đúng** |
| Quy ước đặt tên tiết khí (đầu ngày vs cuối ngày) | 3 | khác quy ước, không ai sai |
| licham365 đặt mốc giao tiết sớm 1 ngày | 4/318 | **ta đúng** |
| licham365 tự mâu thuẫn với danh sách sao của nó | 3 | **ta đúng** |

**Sóc sát nửa đêm.** Tháng 7/2026: nhật thực toàn phần 12/8/2026 lúc 17:46 UTC
= 00:46 ngày 13/8 giờ VN. Đa thức `new_moon()` của Hồ Ngọc Đức (sai số vài phút)
xếp mùng 1 vào 12/8 thay vì 13/8. Đây là giới hạn đã ghi nhận từ giai đoạn 2, và
**rất hiếm**: đo 602 ngày trên 20 tháng khác thì khớp **602/602**; chỉ tháng
7/2026 sai. Kéo theo sai cả Tam Nương/Nguyệt Kỵ vì hai mục này đếm theo ngày Âm.

**Quy ước tên tiết khí.** Ta gọi tên tiết theo trạng thái **cuối ngày** (ngày
chứa khoảnh khắc giao tiết đã mang tên tiết mới); licham365 gọi theo trạng thái
**đầu ngày**. Chỉ ảnh hưởng nhãn hiển thị, **không** ảnh hưởng tháng tiết khí
dùng cho Trực.

**licham365 đặt mốc tiết sớm 1 ngày.** Đo trên 318 ngày, chỉ 4 ngày lệch, và cả
4 đều có mốc tiết thật rơi trong khoảng **01:57–05:16 giờ VN** của ngày kế tiếp:

| Ngày lệch | Mốc tiết thật (giờ VN) |
|---|---|
| 2024-10-07 | Hàn Lộ 8/10 lúc 01:57 |
| 2024-11-06 | Lập Đông 7/11 lúc 05:16 |
| 2025-07-06 | Tiểu Thử 7/7 lúc 03:04 |
| 2026-02-03 | Lập Xuân 4/2 lúc 02:54 |

Ở các mốc rơi muộn hơn trong ngày (Lập Xuân 2025 21:07, Kinh Trập 2025 15:03,
Bạch Lộ 2026 21:41) hai bên khớp nhau. Ta theo quy ước cổ điển: **ngày giao tiết
là ngày Dương lịch chứa khoảnh khắc đó**.

*Bằng chứng licham365 sai ở các ca này*: trên chính những trang đó, dòng "Tiết:"
vẫn ghi tiết **cũ** trong khi Trực đã nhảy sang tháng **mới** — mâu thuẫn nội bộ.

**licham365 tự mâu thuẫn (Chu Tước).** 3 ngày licham365 kết luận "Hoàng đạo
(tốt)" nhưng chính trang đó liệt kê sao "Chu tước hắc đạo". Cả 3 cùng dạng: ngày
Chi **Dậu** trong tháng Âm lịch **4 hoặc 10**. Ta theo vòng 12 Thần: Chu Tước →
Hắc Đạo.

### 5.5 licham365 có 3 kết luận, ta có 2

licham365 chỉ liệt kê **8/12** vị Thần, nên 4 vị còn lại (Thiên Hình, Kim Quỹ,
Thiên Lao, Tư Mệnh) thành **"Bình thường"**. Ta theo cách chia cổ điển **6 tốt /
6 xấu**. Với những ngày licham365 ghi "Bình thường", test **không ràng buộc**
kết luận tốt/xấu — vì đó là khác biệt về cách trình bày của nguồn, không phải
sai số tính toán.

---

## 6. Bug đã sửa trong giai đoạn này

**Hằng số epoch trong `sun_longitude_deg_at_local_midnight` (`astronomy.rs`).**

Hàm này tính `real_jd = jdn − 0.5 − tz/24` (đã trừ nửa ngày), rồi lại dùng epoch
`2451545.5` (vốn đã *gộp sẵn* nửa ngày đó) → **double-count 0.5 ngày** → kinh độ
Mặt Trời thấp hơn thực tế ~0.49° → mọi mốc tiết khí bị đẩy trễ ~0.5 ngày, khiến
khoảng 50% mốc lệch đúng 1 ngày.

Giai đoạn 4 đã *quan sát* thấy triệu chứng ("4/6 mốc Lập Xuân lệch so với HK
Observatory") nhưng quy nhầm cho giới hạn độ chính xác của đa thức Hồ Ngọc Đức.
Thực ra là bug hằng số.

Sửa: `2451545.5` → `2451545.0`. Kết quả sau khi sửa:

- **24/24** tiết khí năm 2024 khớp đúng ngày với lịch vạn niên;
- **10/10** mốc Lập Xuân 2017–2026 khớp đúng ngày (trước khi sửa: 5/10).

Cả hai đều có test regression trong `bat_tu/tiet_khi.rs`.

> ⚠️ `sun_longitude_at_noon` ngay phía trên dùng `2451545.5` là **đúng**, vì ở đó
> shift −0.5 được gộp thẳng vào hằng số chứ không trừ riêng. Hai hàm khác nhau ở
> chỗ đó — **đừng "đồng bộ" hằng số giữa chúng**.

Sửa bug này cũng làm ranh giới tháng/năm Bát Tự (giai đoạn 4) chính xác hơn.

**Đã audit riêng ảnh hưởng ngược tới giai đoạn 3 & 4** — kết quả tóm tắt:

- **Bát Tự bị ảnh hưởng thật**: quét 1960–2030, **1.635% số ngày** cho lá số sai
  (0.135% sai trụ Năm, 1.50% sai trụ Tháng).
- 7 case Bát Tự cũ **không** bị ảnh hưởng, nhưng **nhờ may chứ không phải thiết
  kế** — case5 nằm đúng trên mốc Lập Hạ. Đã bổ sung **11 case biên**, trong đó
  **8 case bị mã cũ tính sai**.
- **Tử Vi không bị ảnh hưởng** (chỉ dùng ngày Âm lịch, đi nhánh epoch đúng) —
  khoá bằng 2 test.
- Kiểm chứng bản sửa với **Đài Thiên văn Hồng Kông**: **192/192**.

Chi tiết: [`bat_tu/README.md`](../bat_tu/README.md) và
[`tests/bug7_epoch_audit.rs`](../../tests/bug7_epoch_audit.rs).

**Tên tiết khí sai trong `TIET_KHI_TABLE`**: 75° ghi "Tiểu Mãn" (đúng: Mang
Chủng) và 105° ghi "Mang Chủng" (đúng: Tiểu Thử). Kinh độ vốn đúng nên tháng Bát
Tự không bị ảnh hưởng — chỉ sai nhãn hiển thị.

---

## 7. API

```rust
use chrono::NaiveDate;
use tinhban_core::danh_gia_ngay;

let a = danh_gia_ngay(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap())?;

a.lunar_date;                    // 6/2/2024
a.day_can_chi;                   // Mậu Dần
a.tiet_khi;                      // "Kinh Trập"
a.hoang_dao_hac_dao.is_hoang_dao;// true
a.hoang_dao_hac_dao.than;        // ThanSat::ThanhLong
a.truc;                          // Truc::Be
a.gio_hoang_dao;                 // 6 khung giờ tốt
a.gio_hac_dao;                   // 6 khung giờ xấu
a.cac_gio;                       // đủ 12 khung, theo thứ tự Tý → Hợi
a.kieng_ky;                      // Vec<KiengKy>, rỗng nếu ngày "sạch"
```

`danh_gia_khoang(from, to)` cho một khoảng ngày (bao gồm cả hai đầu).

Feature `serde` (tắt mặc định) bật `Serialize`/`Deserialize` cho toàn bộ kiểu —
`tinhban-api` bật feature này để trả JSON.

## 8. Phạm vi & giới hạn

- Năm Dương lịch **1900–2100** (giới hạn của lõi âm lịch giai đoạn 2).
- Ngày Âm lịch có thể lệch 1 ngày khi sóc rơi sát nửa đêm giờ VN — hiếm
  (0/602 ngày đo được ngoài tháng 7/2026), xem 5.4.
- Bảng Sát Chủ là **hợp** của nhiều dòng truyền, rộng hơn một bảng đơn lẻ.
- Phần "nên làm / không nên làm" của mỗi Trực là mô tả ngắn gọn theo dân gian;
  diễn giải chi tiết hơn cho từng loại việc do tầng scrape bổ sung.
