#==========================================================================================
# Builder
#==========================================================================================
FROM rust:1.64 AS builder
# Build a trivial project to trigger crates.io index update
WORKDIR /usr/src/tmp
RUN cargo init .
RUN cargo add serde

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

COPY --from=builder /usr/src/iora/target/release/iora_service /usr/local/bin/iora/iora_service
ENTRYPOINT /usr/local/bin/iora/iora_service -p $IORA_PORT