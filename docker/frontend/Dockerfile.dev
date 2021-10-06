FROM rust:1.54

RUN cargo install --force trunk wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown

RUN groupadd --gid 1000 app && \
    useradd --uid 1000 --gid app --system --create-home app

USER app

WORKDIR /app

CMD ["trunk", "serve", "--release"]
