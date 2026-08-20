# Từ điển Tử Vi — nội dung

39 mục markdown: **14 chính tinh + 13 phụ tinh + 12 cung**.

## Phạm vi: chỉ giải nghĩa những gì engine đã implement

Danh sách sao ở đây được **khoá cứng vào code** — test
`tu_dien_phu_het_sao_va_cung_da_implement` đối chiếu từng mục với
`tinhban_core::Sao::ALL` và `PalaceName::all_in_order()`. Thêm sao mới vào enum
mà quên viết mục từ điển thì test đỏ, và ngược lại: viết mục cho sao chưa
implement cũng đỏ.

> ⚠️ Đề bài giai đoạn 6 ghi "14 phụ tinh" nhưng liệt kê ra **13** sao. Code cũng
> có đúng 13 phụ tinh. Từ điển theo code: **13**.

**Chưa có (vì engine chưa implement)**: Địa Không, Địa Kiếp (2 sao còn lại của
Lục Sát), Tứ Hoá (Hoá Lộc / Quyền / Khoa / Kỵ), và các vòng sao phụ khác.

**Cố ý không đưa vào**: 12 sao vòng **Trường Sinh** — engine có tính và UI có
hiển thị, nhưng đề bài giai đoạn này giới hạn phạm vi từ điển ở "14 chính tinh,
phụ tinh, và 12 cung". Để giai đoạn sau nếu cần.

Cũng **không có thuật ngữ Bát Tự** ở giai đoạn này (đúng phạm vi đề bài).

## Nguồn tham khảo

Nội dung được đối chiếu với các nguồn công khai dưới đây, **không tự bịa ý nghĩa
sao**:

- [Học viện Lý số — 14 chính tinh](https://hocvienlyso.org/14-chinh-tinh.html)
- [4T Human — Tổng quan về 14 chính tinh](https://4thuman.com/huyen-hoc-phuong-dong/tu-vi/14-chinh-tinh-tong-quan/)
- [Tử Vi Lý Số — thảo luận âm dương / Nam–Bắc Đẩu](https://tuvilyso.org/forum/topic/20646-su-khac-nhau-cua-tinh-am-duong-va-nam-bac-cua-tinh-dau/)
- [Tử Vi Đẩu Số Toàn Thư (bản PDF công khai)](https://thuviensach.vn/img/pdf/9439-do-giai-tu-vi-dau-so-toan-thu-thuviensach.vn.pdf)

## Nguồn mâu thuẫn nhau ở đâu — và đã chọn thế nào

Đúng tinh thần của dự án: khi nguồn không thống nhất thì **ghi rõ ra** thay vì
chọn im lặng.

| Mục | Bất đồng giữa các nguồn | Từ điển này chọn |
|---|---|---|
| Phân nhóm Bắc/Nam Đẩu | hocvienlyso.org xếp Thái Dương, Vũ Khúc, Liêm Trinh vào **Nam Đẩu** và Thiên Đồng, Thiên Phủ, Thái Âm, Tham Lang, Cự Môn, Thiên Lương, Phá Quân vào **Bắc Đẩu** — mâu thuẫn với cách chia cổ điển | Theo **cách chia cổ điển** (4thuman.com xác nhận): Bắc Đẩu 6 sao (Tử Vi, Vũ Khúc, Liêm Trinh, Tham Lang, Cự Môn, Phá Quân), Nam Đẩu 6 sao (Thiên Phủ, Thiên Cơ, Thiên Lương, Thiên Đồng, Thiên Tướng, Thất Sát), Trung Thiên 2 sao (Thái Dương, Thái Âm) |
| Tử Vi âm hay dương | Can hệ cổ điển: **kỷ Thổ (Âm)**. Nhiều tài liệu Việt phổ thông: **Dương** | Theo can hệ cổ điển → **Âm** (đã ghi chú ngay trong mục) |
| Tham Lang hành gì | **giáp Mộc (Dương Mộc)** vs **Thủy** / "Thủy đới Mộc" | **Mộc** (đã ghi chú) |
| Thiên Lương hành gì | **Mộc** (tài liệu Việt) vs **mậu Thổ** (can hệ) vs "Mộc đới Thổ" | **Mộc** (đã ghi chú) |
| Thất Sát hành gì | Cổ thư ghi "**Kim đới Hoả**" | **Kim** làm hành chính (đã ghi chú) |

Bốn mục có ghi chú "Lưu ý về thuộc tính" ngay trong nội dung là các mục thuộc
bảng trên — người đọc thấy được ngay là chỗ đó có tranh luận.

## Định dạng file

Mỗi mục là một file `.md` với frontmatter:

```markdown
---
slug: sao-tu-vi          # khoá tra cứu + URL; PHẢI khớp Sao::slug() / PalaceName::slug()
title: Tử Vi
kind: chinh-tinh         # chinh-tinh | phu-tinh | cung
nhom: Bắc Đẩu            # Bắc Đẩu | Nam Đẩu | Trung Thiên | Lục Cát | Lục Sát | 12 cung | ...
nguhanh: Thổ             # rỗng với cung
amduong: Âm              # rỗng với cung
aliases: Đế tinh, Đế toạ # tên gọi khác, dùng cho tìm kiếm
---

nội dung markdown…
```

`slug` là **khoá dữ liệu**, không phải nhãn hiển thị — đổi slug sẽ làm hỏng link
đã lưu và làm test đối chiếu với code đỏ.

## Nội dung này được nạp vào app thế nào

Toàn bộ thư mục được **nhúng thẳng vào binary** lúc biên dịch (`include_dir!`),
nên deploy chỉ cần copy đúng một file binary — không phải mang theo `content/`.

Muốn sửa nội dung mà không build lại: đặt biến môi trường
`TINHBAN_CONTENT_DIR=/đường/dẫn/tới/content/tu-dien`, app sẽ đọc từ đĩa thay vì
dùng bản nhúng. Tiện khi soạn thảo.

Mỗi lần khởi động, app xoá sạch bảng từ điển rồi nạp lại từ đầu — nội dung nhỏ
(39 mục) nên rẻ, và tránh được tình trạng bản cũ còn sót sau khi sửa file.
