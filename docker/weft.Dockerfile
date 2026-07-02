# ──────────────────────────────────────────────────────────────────
# Weft Base Builder
# Kompiliert das gesamte Weft-Projekt (mit unseren Custom Catalog-Nodes)
# ──────────────────────────────────────────────────────────────────
FROM rust:1.85-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev curl bash git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Layer 1: Dependencies (fuer Docker Cache)
COPY weft/Cargo.toml weft/Cargo.lock* ./
COPY weft/crates/weft-core/Cargo.toml   crates/weft-core/
COPY weft/crates/weft-nodes/Cargo.toml  crates/weft-nodes/
COPY weft/crates/weft-api/Cargo.toml    crates/weft-api/
COPY weft/crates/weft-orchestrator/Cargo.toml crates/weft-orchestrator/

RUN mkdir -p crates/weft-core/src crates/weft-nodes/src crates/weft-api/src crates/weft-orchestrator/src && \
    echo 'fn main() {}' > crates/weft-core/src/lib.rs && \
    echo 'fn main() {}' > crates/weft-nodes/src/main.rs && \
    echo 'fn main() {}' > crates/weft-nodes/src/lib.rs && \
    echo 'fn main() {}' > crates/weft-api/src/main.rs && \
    echo 'fn main() {}' > crates/weft-orchestrator/src/main.rs && \
    cargo build --release 2>/dev/null || true

# Layer 2: Node Registry Dependencies
COPY weft/catalog/ catalog/
COPY scripts/catalog-link.sh scripts/
RUN bash scripts/catalog-link.sh --copy || true
COPY crates/ crates/

# Layer 3: Full Build
RUN cargo build --release

# ──────────────────────────────────────────────────────────────────
# Orchestrator Runtime
# ──────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS orchestrator
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/orchestrator /usr/local/bin/
EXPOSE 9080
ENV RESTATE_URL=http://restate:8080
CMD ["orchestrator"]

# ──────────────────────────────────────────────────────────────────
# API Runtime
# ──────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS api
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/weft-api /usr/local/bin/
EXPOSE 3000
ENV RESTATE_URL=http://restate:8080
CMD ["weft-api"]

# ──────────────────────────────────────────────────────────────────
# Node Runner Runtime
# ──────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS node-runner
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/node-runner /usr/local/bin/
EXPOSE 9082
ENV ORCHESTRATOR_URL=http://restate:8080
CMD ["node-runner"]
