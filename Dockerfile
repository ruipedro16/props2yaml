FROM rust:1.90-bookworm AS builder

WORKDIR /build

COPY --from=golang:1.25-bookworm /usr/local/go/ /usr/local/go/

ENV PATH="/usr/local/go/bin:${PATH}"

RUN go install github.com/google/yamlfmt/cmd/yamlfmt@latest

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        wget \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/props2yaml /usr/local/bin/props2yaml
COPY --from=builder /root/go/bin/yamlfmt /usr/local/bin/yamlfmt

RUN useradd -m -u 1000 user \
    && chown -R user:user /usr/local/bin/props2yaml /usr/local/bin/yamlfmt

USER user

WORKDIR /props2yml

ENTRYPOINT ["props2yaml"]
CMD ["--help"]
