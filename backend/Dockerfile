FROM rust:1.93.1 as builder

WORKDIR /app

# Khuyến nghị: build cache dependencies riêng để build nhanh hơn, nhưng đây là cấu trúc gọn
COPY . .
RUN cargo build --release

# Dùng debian làm base image để chứa các CA certificate (Cần HTTPS để gọi Cloudinary)
FROM debian:bookworm-slim

WORKDIR /app

# Cài đặt chứng chỉ SSL/TLS để reqwest có thể call ra ngoài an toàn
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rewrite .

# Tạo folder uploads nếu chạy dạng lưu file cục bộ thay vì Cloudinary
RUN mkdir -p uploads

# Port server AppChat lắng nghe
EXPOSE 8080

CMD ["./rewrite"]
