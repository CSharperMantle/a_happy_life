FROM rust:1.74.1-slim-bullseye as builder
WORKDIR /usr/src/a_happy_life
COPY . .
RUN mkdir install
RUN cargo install --path . --root /usr/src/a_happy_life/install

FROM rust:1.74.1-slim-bullseye
ARG USERNAME=a_happy_life
ARG USER_UID=1000
ARG USER_GID=$USER_UID
EXPOSE 10000
COPY --from=builder /usr/src/a_happy_life/install/bin/a_happy_life /usr/local/bin/a_happy_life
COPY --from=builder /usr/src/a_happy_life/docker/xinetd.a_happy_life.conf /usr/local/etc/xinetd.a_happy_life.conf
COPY --from=builder /usr/src/a_happy_life/docker/sources.tuna.list /etc/apt/sources.list
RUN groupadd --gid $USER_GID $USERNAME && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME
RUN apt-get update && apt-get install -y xinetd
RUN rustup component add clippy
CMD ["script", "-c", "xinetd -dontfork -f /usr/local/etc/xinetd.a_happy_life.conf -d"]
