# syntax=docker/dockerfile:1

# Install frontend dependencies on Linux so native Vite/esbuild packages match
# the builder, rather than copying node_modules from the host.
FROM node:22.23.4-alpine3.23 AS frontend-deps
WORKDIR /workspace/frontend
RUN npm install --global pnpm@10.11.0
COPY frontend/package.json frontend/pnpm-lock.yaml ./
RUN --mount=type=cache,target=/pnpm/store,sharing=locked \
    pnpm install --frozen-lockfile --store-dir=/pnpm/store

FROM rust:1.99.1-alpine3.23 AS build
WORKDIR /workspace
# Node's C++/atomic runtime and readelf are build-time tools only.
RUN apk add --no-cache libstdc++ libatomic binutils
# Cargo's build.rs runs Vite and embeds its output. Only the Node executable is
# needed here; pnpm and npm stay in the dependency stage.
COPY --from=frontend-deps /usr/local/bin/node /usr/local/bin/node
COPY frontend/ ./frontend/
COPY --from=frontend-deps /workspace/frontend/node_modules ./frontend/node_modules
COPY backend/ ./backend/

# This is a public browser key, baked into the frontend bundle at build time.
ARG VITE_GOOGLE_MAPS_API_KEY=""
ENV VITE_GOOGLE_MAPS_API_KEY=${VITE_GOOGLE_MAPS_API_KEY}
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/workspace/backend/target,sharing=locked \
    target="$(rustc -vV | sed -n 's/^host: //p')" \
    && RUSTFLAGS="-C target-feature=+crt-static" \
       cargo build --release --locked --target "$target" --manifest-path backend/Cargo.toml \
    && install -m 755 "backend/target/$target/release/backend" /usr/local/bin/stadt-land-streetview \
    && readelf --program-headers --dynamic /usr/local/bin/stadt-land-streetview > /tmp/app-elf.txt \
    && if grep -Eq 'INTERP|NEEDED' /tmp/app-elf.txt; then \
         echo "The scratch runtime requires a static executable without a dynamic loader or shared libraries." >&2; \
         exit 1; \
       fi

# Both the frontend and API are contained in the executable. The runtime image
# has no Node, Rust toolchain, source checkout, or external static-assets folder.
FROM scratch AS runtime
WORKDIR /app
COPY --from=build /usr/local/bin/stadt-land-streetview /usr/local/bin/stadt-land-streetview
COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENV SLS_BIND_ADDR=0.0.0.0:3000
USER 65532:65532
EXPOSE 3000
# The server's existing Ctrl+C handler closes WebSockets and shuts Axum down.
STOPSIGNAL SIGINT
ENTRYPOINT ["/usr/local/bin/stadt-land-streetview"]
