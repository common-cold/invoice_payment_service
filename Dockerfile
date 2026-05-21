FROM rust:1.85 as builder

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./
COPY common ./common
COPY db ./db
COPY backend ./backend

# Build the project
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /app/target/release/backend /app/backend
COPY --from=builder /app/target/release/db /app/db
COPY --from=builder /app/target/release/common /app/common

# Copy migrations
COPY db/migrations ./db/migrations

# Install sqlx-cli for migrations
RUN cargo install sqlx-cli --no-default-features --features postgres

WORKDIR /app

ENV RUST_BACKTRACE=1
ENV DATABASE_URL=postgresql://postgres:postgres@database:5432/invoice_payment

CMD ["/app/backend"]
