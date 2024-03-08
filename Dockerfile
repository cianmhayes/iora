#==========================================================================================
# Builder
#==========================================================================================
FROM rust:1.64-bullseye AS builder
RUN cargo install cargo-audit

# Build iora
WORKDIR /usr/src/iora
COPY . .
RUN cargo build --release --locked

#==========================================================================================
# Service
#==========================================================================================
FROM debian:bullseye-slim AS runtime
WORKDIR /usr/local/bin/iora
ARG IORA_PORT=3000
EXPOSE ${IORA_PORT}
ENV IORA_PORT=${IORA_PORT}

RUN apt-get update
RUN apt-get install -y openssl ca-certificates

COPY --from=builder /usr/src/iora/target/release/iora_service /usr/local/bin/iora/iora_service
COPY --from=builder /usr/src/iora/iora_service/config/* /usr/local/bin/iora/config/
ENTRYPOINT /usr/local/bin/iora/iora_service -p $IORA_PORT