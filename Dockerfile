# Build stage
FROM rust:1.85-bookworm AS builder

WORKDIR /app

# Install build dependencies including Oracle Instant Client
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev wget unzip libaio-dev && \
    rm -rf /var/lib/apt/lists/*

# Download and install Oracle Instant Client for build stage
RUN mkdir -p /opt/oracle && \
    cd /tmp && \
    wget https://download.oracle.com/otn_software/linux/instantclient/2110000/instantclient-basic-linux.x64-21.10.0.0.0dbru.zip && \
    unzip instantclient-basic-linux.x64-21.10.0.0.0dbru.zip && \
    mv instantclient_21_10 /opt/oracle/ && \
    rm instantclient-basic-linux.x64-21.10.0.0.0dbru.zip && \
    echo /opt/oracle/instantclient_21_10 > /etc/ld.so.conf.d/oracle-instantclient.conf && \
    ldconfig

# Set Oracle environment variables for build
ENV LD_LIBRARY_PATH=/opt/oracle/instantclient_21_10
ENV ORACLE_HOME=/opt/oracle/instantclient_21_10

# Copy manifests first for better caching
COPY Cargo.toml ./

# Generate Cargo.lock if it doesn't exist
RUN cargo fetch

# Copy source code
COPY src ./src

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies including Oracle Instant Client
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 wget unzip libaio-dev && \
    rm -rf /var/lib/apt/lists/*

# Download and install Oracle Instant Client for runtime
RUN mkdir -p /opt/oracle && \
    cd /tmp && \
    wget https://download.oracle.com/otn_software/linux/instantclient/2110000/instantclient-basic-linux.x64-21.10.0.0.0dbru.zip && \
    unzip instantclient-basic-linux.x64-21.10.0.0.0dbru.zip && \
    mv instantclient_21_10 /opt/oracle/ && \
    rm instantclient-basic-linux.x64-21.10.0.0.0dbru.zip && \
    echo /opt/oracle/instantclient_21_10 > /etc/ld.so.conf.d/oracle-instantclient.conf && \
    ldconfig

# Set Oracle environment variables for runtime
ENV LD_LIBRARY_PATH=/opt/oracle/instantclient_21_10
ENV ORACLE_HOME=/opt/oracle/instantclient_21_10 

# Copy binary from builder
COPY --from=builder /app/target/release/interhealth-api /usr/local/bin/interhealth-api

# Copy .env if needed (or use environment variables)
# COPY .env /app/.env

WORKDIR /app

EXPOSE ${APP_PORT}

CMD ["interhealth-api"]