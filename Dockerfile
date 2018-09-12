FROM rust:1.28.0

WORKDIR /usr/src/gpu-monitor
COPY . .

RUN cargo install

CMD ["gpu-monitor"]
