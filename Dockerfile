FROM rust:1.74.1-alpine3.18 as builder
WORKDIR /usr/src/a_happy_life
COPY . .
RUN mkdir install
RUN cargo install --path . --root /usr/src/a_happy_life/install

FROM alpine:3.18
EXPOSE 10000
RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.tuna.tsinghua.edu.cn/g' /etc/apk/repositories
RUN apk update && apk add rust rust-clippy s6-networking
COPY --from=builder /usr/src/a_happy_life/install/bin/a_happy_life /usr/local/bin/a_happy_life
CMD ["s6-tcpserver", "-v", "-c", "10", "0.0.0.0", "10000", "/usr/local/bin/a_happy_life"]
