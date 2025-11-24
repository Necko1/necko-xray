FROM rust:alpine3.21 AS builder
WORKDIR /app

RUN apk add --no-cache musl-dev

RUN mkdir src && echo "fn main() {}" > src/main.rs
COPY Cargo.toml .
RUN cargo build --release

COPY src ./src
RUN touch src/main.rs && cargo build --release


FROM alpine:3.22
LABEL authors="necko"

# https://github.com/XTLS/Xray-core/releases
ARG XRAY_VERSION=v25.10.15

RUN apk add --no-cache curl unzip ca-certificates netcat-openbsd && \
    mkdir -p /var/log/xray /usr/local/share/xray

RUN curl -L -H "Cache-Control: no-cache" -o xray.zip https://github.com/XTLS/Xray-core/releases/download/${XRAY_VERSION}/Xray-linux-64.zip && \
    unzip xray.zip && \
    mv xray /usr/local/bin/xray && \
    mv geoip.dat /usr/local/share/xray/ && \
    mv geosite.dat /usr/local/share/xray/ && \
    chmod +x /usr/local/bin/xray && \
    rm xray.zip && \
    apk del curl unzip

ENV XRAY_LOCATION_ASSET=/usr/local/share/xray/

COPY --from=builder /app/target/release/necko-xray /usr/local/bin/necko-xray

CMD ["/usr/local/bin/necko-xray", "daemon"]
