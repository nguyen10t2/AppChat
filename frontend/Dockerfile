FROM oven/bun:latest as build

WORKDIR /app

# Copy lockfile và package.json để cài dependencies tĩnh trước
COPY package.json bun.lockb* ./
RUN bun install --frozen-lockfile || bun install

COPY . .

# Build Vite / React 19 ra thư mục dist
RUN bun run build

FROM nginx:alpine

# Copy config của Nginx để bọc Single Page Application (Push State Router)
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Bê file đã build từ build stage
COPY --from=build /app/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
