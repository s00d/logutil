# Select distro
ARG FROM_DISTRO=bullseye

FROM php8.1-fpm-${FROM_DISTRO}

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN apt-get update && apt install curl build-essential gcc libclang-dev make openssl libssl-dev git -y

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /code
ENTRYPOINT [ "" ]