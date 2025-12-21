#stage 1
FROM rust:1.91 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release


#stage 2
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/personal-wiki .
COPY --from=builder /app/pages/index.html /app/pages/index.html
COPY --from=builder /app/scripts/script.js /app/scripts/script.js
EXPOSE 3000
CMD ["./personal-wiki"]