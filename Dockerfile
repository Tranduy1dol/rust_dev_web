FROM rust:1.76 AS builder

RUN apt -y update
RUN apt install libssl-dev
RUN #apt install libudev

WORKDIR /app

COPY ./ .

ENV RUSTFLAGS='-C linker=x86_64-linux-gnu-gcc'
ENV CC='gcc'
ENV CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc
ENV CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc

RUN cargo build --target x86_64-unknown-linux-gnu --release

FROM scratch

WORKDIR /app

#COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/restful-api ./
#COPY --from=builder /app/.env ./
COPY ./ .

CMD ["/app/restful-api"]
