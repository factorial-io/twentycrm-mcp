FROM rust:1-bookworm AS builder

# Third-party license notices (MIT/BSD/Apache require shipping them with binaries).
ARG CARGO_ABOUT_VERSION=0.9.3
ARG TARGETARCH
RUN case "$TARGETARCH" in \
      arm64) arch=aarch64; sha=eb4d9ff6ebd2518152a8308729431a04b3836e71d2bd5ead7aea69f91a71162e ;; \
      *)     arch=x86_64;  sha=3ad1e94ea1e63d69d1961f4fb15ccbbe9575d906111f96d2cb4dce6df48f71b4 ;; \
    esac \
    && name="cargo-about-${CARGO_ABOUT_VERSION}-${arch}-unknown-linux-musl" \
    && curl -fsSL -o /tmp/ca.tar.gz "https://github.com/EmbarkStudios/cargo-about/releases/download/${CARGO_ABOUT_VERSION}/${name}.tar.gz" \
    && echo "${sha}  /tmp/ca.tar.gz" | sha256sum -c - \
    && tar xzf /tmp/ca.tar.gz -C /tmp \
    && mv "/tmp/${name}/cargo-about" /usr/local/cargo/bin/ \
    && rm -rf /tmp/ca.tar.gz "/tmp/${name}"

WORKDIR /app
COPY Cargo.toml Cargo.lock about.toml about.hbs LICENSE ./
COPY src/ src/

RUN cargo build --release \
    && cargo about generate about.hbs -o THIRD_PARTY_LICENSES.html

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/twentycrm-mcp /usr/local/bin/twentycrm-mcp
COPY --from=builder /app/LICENSE /app/THIRD_PARTY_LICENSES.html /usr/share/doc/twentycrm-mcp/

EXPOSE 8080

CMD ["twentycrm-mcp"]
