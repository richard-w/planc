# Build image.
FROM rust AS build
# Install musl toolchain.
RUN rustup target add x86_64-unknown-linux-musl
# Install wasm toolchain.
RUN rustup target add wasm32-unknown-unknown
# Install trunk
RUN cargo install trunk
# Add unprivileged user
RUN groupadd user && useradd -m -g user user
# Copy sources
ADD --chown=user:user . /work/
# Build backend
WORKDIR /work
RUN cargo build --target x86_64-unknown-linux-musl --release

# Runtime image.
FROM alpine
# Copy application.
COPY --from=build /work/target/x86_64-unknown-linux-musl/release/planc /
# Run as unprivileged user.
RUN adduser -D user
USER user:user
# Maximum sessions on this instance
ENV MAX_SESSIONS=8
# Maximum users in a session
ENV MAX_USERS=16
# Set entrypoint to run application.
ENV RUST_LOG=info
EXPOSE 8080/tcp
ENTRYPOINT /planc \
    --bind-address 0.0.0.0 \
    --bind-port 8080 \
    --max-sessions ${MAX_SESSIONS} \
    --max-users ${MAX_USERS}
