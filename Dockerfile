FROM rust:1.54

RUN cargo install --force cargo-make

RUN groupadd --gid 1000 app && \
    useradd --uid 1000 --gid app --system app

USER app

WORKDIR /app

CMD ["tail", "-f", "/dev/null"]
