# Frontend build image.
FROM node:16
# Install angular CLI.
RUN npm install -g @angular/cli
# Add user and group.
USER node:node
# Copy package info.
ADD --chown=node:node web/package.json web/package-lock.json /work/
# Set working directory.
WORKDIR /work
# Install dependencies.
RUN npm install
# Copy sources.
ADD --chown=node:node web /work
# Build frontend.
RUN ng build

# Backend build image.
FROM rust
# Install musl toolchain.
RUN rustup target add x86_64-unknown-linux-musl
# Add unprivileged user.
RUN groupadd user && useradd -m -g user user
# Copy sources.
ADD --chown=user:user Cargo.lock Cargo.toml /work/
ADD --chown=user:user src /work/src/
# Copy frontend build.
COPY --from=0 --chown=user:user /work/dist /work/web/dist
# Build backend
USER user:user
WORKDIR /work
RUN cargo build --target x86_64-unknown-linux-musl --release 

# Runtime image.
FROM alpine
# Copy application.
COPY --from=1 /work/target/x86_64-unknown-linux-musl/release/planc /
# Run as unprivileged user.
RUN adduser -D user
USER user:user
# Set entrypoint to run application.
ENV RUST_LOG=info
EXPOSE 8080/tcp
ENTRYPOINT /planc --bind-address 0.0.0.0 --bind-port 8080
