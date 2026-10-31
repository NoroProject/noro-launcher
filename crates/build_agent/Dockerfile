# Linux build agent. GPUI's system libraries, the same ones release-launcher.yml
# installs, plus rustup: the toolchain pinned in rust-toolchain.toml installs
# itself on the first build.
#
#   docker build -f crates/build_agent/Dockerfile -t noro-build-agent .
#   docker run -d --restart unless-stopped --name noro-build-agent-arm64 \
#     -e NORO_MASTER_URL -e NORO_ADMIN_TOKEN \
#     -v noro-build-agent-arm64:/root/.noro-build-agent \
#     noro-build-agent run --name <machine>-linux-arm64
#
# Built and run with --platform linux/amd64 the same image builds
# x86_64-unknown-linux-gnu; on an Apple Silicon Mac that goes through emulation
# and is slow, but only the first build is cold — the volume keeps target/.
FROM rust:1-bookworm AS agent
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p build_agent

FROM rust:1-bookworm
RUN apt-get update \
 && apt-get install -y --no-install-recommends pkg-config \
      libxkbcommon-dev libxkbcommon-x11-dev libfontconfig-dev libfreetype-dev \
 && rm -rf /var/lib/apt/lists/*
COPY --from=agent /src/target/release/noro-build-agent /usr/local/bin/
ENTRYPOINT ["noro-build-agent"]
CMD ["run"]
