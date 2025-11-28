# syntax=docker/dockerfile:1
FROM rust:1.76 as builder
WORKDIR /workspace
COPY . .
RUN cargo build -p noa_ui_api --release

FROM debian:bookworm-slim
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*
COPY --from=builder /workspace/target/release/noa-ui-api-server /usr/local/bin/noa-ui-api-server
EXPOSE 8787 50051
ENV NOA_UI_API_ADDR=0.0.0.0:8787
ENV NOA_UI_API_GRPC_ADDR=0.0.0.0:50051
CMD ["/usr/local/bin/noa-ui-api-server", "--http-addr", "0.0.0.0:8787", "--grpc-addr", "0.0.0.0:50051"]
