# =========================
# 1️⃣ Builder stage
# =========================
FROM --platform=linux/amd64 ubuntu:22.04 AS builder

# ---- system deps ----
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    cmake \
    clang \
    && rm -rf /var/lib/apt/lists/*

# ---- install Rust ----
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y \
    && rustup default stable

# ---- workspace ----
WORKDIR /zkcg

# ---- copy full monorepo ----
COPY . .

# ---- build ONLY api crate ----
RUN cargo build -p api --release --features "zk-halo2"

# =========================
# 2️⃣ Runtime stage
# =========================
FROM --platform=linux/amd64 ubuntu:22.04

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /zkcg

# host binary
COPY --from=builder /zkcg/target/release/api ./api

ENV PORT=8080
ENV ZKCG_ENABLE_PROTOCOL=1
ENV ZKCG_STATE_BACKEND=sqlite
ENV ZKCG_STATE_PATH=/data/protocol-state.db
RUN mkdir -p /data
VOLUME ["/data"]
EXPOSE 8080

CMD ["./api"]
