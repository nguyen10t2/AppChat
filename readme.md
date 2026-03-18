# ⚡ AppChat Rewrite

AppChat Rewrite là một ứng dụng nhắn tin thời gian thực (Real-time Chat Application) đa nền tảng, được viết lại hoàn toàn với kiến trúc hiện đại, tập trung vào hiệu năng tối đa (Rust) và trải nghiệm người dùng mượt mà nhất (React + Optimistic UI).

## 💡 Các Tính Năng Nổi Bật

- **Real-time Messaging:** Nhắn tin tức thời nhờ công nghệ WebSocket (`actix-ws`), hỗ trợ chat cá nhân (Nhắn tin 1-1) và chat nhóm.
- **Trạng thái Online/Offline (Presence):** Theo dõi trạng thái hoạt động của bạn bè thông qua Redis Pub/Sub kết hợp WebSocket.
- **Typing Indicator:** Hiển thị "Đang nhập..." khi đối phương gõ phím.
- **Optimistic UI:** Xây dựng cơ chế cập nhật giao diện trước khi API phản hồi, triệt tiêu độ trễ mạng để có trải nghiệm gõ như Messenger/Zalo.
- **Upload File/Media:** Tích hợp `reqwest` tải ảnh và file đính kèm trực tiếp lên hệ sinh thái **Cloudinary**. Hỗ trợ ảnh, video, tài liệu, file nén.
- **Auth (JWT):** Bảo mật qua 2 lớp (Access Token lưu trên memory, Refresh Token lưu trên HttpOnly Cookies). Password được mã hóa bằng thuật toán `Argon2`.
- **Thiết kế tinh gọn:** Giao diện tối giản, tối sang trọng bằng **TailwindCSS v4**, trang bị **Shadcn UI** và **Framer Motion / Radix**.

---

## 🛠️ Tech Stack

Dự án được cắt làm 2 module riêng biệt:

### 🦀 Backend (Rust / Actix-Web)
- **Core Framework:** Rust + Actix-Web 4.
- **Database:** PostgreSQL (truy vấn với `sqlx` async).
- **Cache & Pub/Sub:** Redis (kết nối qua `deadpool-redis`).
- **WebSocket:** `actix-ws` kết hợp với `DashMap` + `Rayon` xử lý đồng thời hàng nghìn kết nối không bị nghẽn (actorless architecture).
- **Security:** `jsonwebtoken`, `argon2`, `validator`.
- **Infrastructure:** `.env` variables, `Cloudinary` API.

### 🌐 Frontend (React 19 / TypeScript)
- **Core:** React 19 + TypeScript + Vite.
- **State Management:** `zustand` (chống re-render vòng lặp với `useShallow`).
- **Routing:** React Router v7.
- **Styling:** TailwindCSS v4, Class Variance Authority (CVA), `clsx`.
- **UI Components:** Shadcn UI, Radix UI Primitives, Phosphor Icons / Lucide.
- **Form & Validation:** React Hook Form + Zod.
- **Network / API:** Axios + Interceptors cho Auto Refresh Token.

---

## 🚀 Hướng Dẫn Cài Đặt (Local Development)

### 1. Yêu Cầu Hệ Thống
- Đã cài đặt [Docker Desktop](https://www.docker.com/products/docker-desktop/) (Hoặc Docker Engine trên Linux).
- Không cần cài PostgreSQL/Redis trên máy tính, mọi thứ đã được nhúng chung vào Docker Compose!

### 2. Thiết Lập Môi Trường (Cho cả Backend lẫn Frontend)
Tại gốc của thư mục `AppChat-Rewrite`, bạn tạo một thư mục `.env` chung cho backend:
```bash
nano backend/.env
```

Và khai báo các Secret (Được Docker đọc tự động):
```env
# Cấu hình Database cho vùng chứa PostgreSQL
POSTGRES_USER=appchat
POSTGRES_PASSWORD=MatKhauBaoMat123
POSTGRES_DB=appchat

# Security
SECRET_KEY=mot_chuoi_bi_mat_rat_la_dai_danh_cho_jwt
ACCESS_TOKEN_EXPIRATION=900       # 15 phút
REFRESH_TOKEN_EXPIRATION=2592000  # 30 ngày

# Media Upload (Cloudinary) - Thay bằng token thật của bạn
CLOUDINARY_URL=cloudinary://api_key:api_secret@cloud_name
```

Tiếp theo, tạo file `.env` cho Frontend:
```bash
nano frontend/.env
```

```env
# URL API gọi từ client (Lên Production thì thay bằng Tên Miền hoặc IP Public)
VITE_API_BASE_URL=http://localhost:8080/api
VITE_WS_URL=ws://localhost:8080/ws
```

### 3. Deploy Toàn Bộ Hệ Thống Với 1 Lệnh Duy Nhất
Đứng ở thư mục gốc của project, gõ:
```bash
docker compose up -d --build
```
Hệ thống sẽ tự động tải Image Postgres, Redis, tự động build Frontend bằng Bun và chạy Backend bằng Rust. **(Backend Rust cũng sẽ tự động kích hoạt tiến trình SQLX Migrations tạo Table ngay khi bật lên!)**

*Server Backend mặc định chạy tại: `http://localhost:8080`*
*Trang Frontend mở ở: `http://localhost:3000`*

---

## 🏗 Kiến Trúc WebSocket

Để giải quyết vấn đề hiệu năng với chuẩn WebSocket do Actix-Web cung cấp, hệ thống hiện dùng thiết kế **Actorless WebSocket** dựa vào `mpsc::channel` và `DashMap`:

1. **Authentication:** Trình duyệt mở TCP Request bắt tay nâng cấp lên WebSocket, client sẽ truyền object `Auth { token }`.
2. **Quản lý Session:** Backend dùng `Uuid v7` định danh Session. Các Session này được nạp vào Map theo cấu trúc `UserId -> Set[SessionId]`. Một User có thể login n Tabs độc lập.
3. **Phân phối:** Khi cần bắn sự kiện (Tin nhắn mới, Xóa tin nhắn, Đang gõ...), Service layer tìm đúng danh sách Users liên quan (Gồm người nhận và CẢ người gửi để đồng bộ multi-tab). Sau đó dùng `Rayon (par_iter)` gọi `tx.send` đa luồng.
4. **Auto-Cleanup:** Việc rác (dọn cache) sinh ra khi client mất mạng cực tối ưu vì Socket đóng sẽ nhả Channel, kích hoạt hàm Drop session tự động không làm kẹt luồng logic.

---

## 📝 Giấy phép
Dự án nội bộ được viết để phục vụ mục đích nghiên cứu thiết kế ứng dụng Real-time hiệu năng cao bằng Rust.
