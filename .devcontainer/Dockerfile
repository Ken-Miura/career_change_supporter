FROM ubuntu:22.04

RUN apt-get update && \
    export DEBIAN_FRONTEND=noninteractive && \
    apt-get install -y \
        build-essential \
        curl \
        git \
        libssl-dev \
        lldb \
        pkg-config \
        vim

ARG USERNAME=developer
ARG GROUPNAME=developer
ARG UID=1000
ARG GID=1000

RUN groupadd -g ${GID} ${GROUPNAME} && \
    useradd -m -s /bin/bash -u ${UID} -g ${GID} ${USERNAME}

USER ${USERNAME}
ENV HOME=/home/${USERNAME}

RUN mkdir ${HOME}/workspace
WORKDIR ${HOME}/workspace

ARG RUST_VERSION=1.65.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION}
ENV PATH=${PATH}:${HOME}/.cargo/bin
ARG SEA_ORM_CLI_VERSION=0.11.2
RUN cargo install --locked --version ${SEA_ORM_CLI_VERSION} sea-orm-cli

ARG NVM_VERSION=0.37.2
RUN curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v${NVM_VERSION}/install.sh | bash
ENV NVM_DIR=${HOME}/.nvm
ARG NODE_JS_VERSION=18.12.1
RUN . ${NVM_DIR}/nvm.sh && \
    nvm install ${NODE_JS_VERSION} && \
    nvm use ${NODE_JS_VERSION}
