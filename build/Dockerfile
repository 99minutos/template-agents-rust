####################################################################################################
## Builder Stage
####################################################################################################
FROM rust:bookworm AS builder
RUN update-ca-certificates

ENV USER=app
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

# RUN apt-get update && \
#    apt-get install -y protobuf-compiler jq && \
#    rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY . .

RUN cargo build --release


RUN SOURCE_BINARY_PATH="/app/target/release/service" && \
    DEST_BINARY_PATH="/app/service" && \
    echo "Copying binary from ${SOURCE_BINARY_PATH} to ${DEST_BINARY_PATH}" && \
    cp "${SOURCE_BINARY_PATH}" "${DEST_BINARY_PATH}" && \
    echo "Stripping binary at ${DEST_BINARY_PATH}" && \
    strip -s "${DEST_BINARY_PATH}" || \
    (echo "Error: Failed during binary processing (copy or strip)." >&2 && exit 1)

####################################################################################################
## Final image Stage
####################################################################################################
FROM gcr.io/distroless/cc-debian12
USER app:app
WORKDIR /app

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

COPY --from=builder /app/service ./service

CMD ["/app/service"]

