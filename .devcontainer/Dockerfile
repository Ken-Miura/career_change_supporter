FROM ubuntu:22.04 as development

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

RUN export RUST_VERSION=1.70.0 && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.70.0
ENV PATH=${PATH}:${HOME}/.cargo/bin
RUN export SEA_ORM_CLI_VERSION=0.11.3 && \
    cargo install --locked --version ${SEA_ORM_CLI_VERSION} sea-orm-cli

RUN export NVM_VERSION=0.39.3 && \
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v${NVM_VERSION}/install.sh | bash
ENV NVM_DIR=${HOME}/.nvm
RUN export NODE_JS_VERSION=18.16.0 && \
    . ${NVM_DIR}/nvm.sh && \
    nvm install ${NODE_JS_VERSION} && \
    nvm use ${NODE_JS_VERSION}

FROM development as server-test-and-build

# ビルドコンテキストはserverディレクトリ直下であると仮定
COPY . .
RUN cargo fmt --check && \
    cargo clippy --all-targets --all-features -- -D warnings && \
    cargo test && \
    cargo build --release

FROM ubuntu:22.04 as user-service

RUN useradd -m -s /bin/bash -U user-service-operator
USER user-service-operator
RUN mkdir -p /home/user-service-operator/workspace
WORKDIR /home/user-service-operator/workspace
COPY --from=server-test-and-build /home/developer/workspace/target/release/user_service ./
EXPOSE 3000
CMD [ "./user_service" ]

FROM ubuntu:22.04 as admin-service

RUN useradd -m -s /bin/bash -U admin-service-operator
USER admin-service-operator
RUN mkdir -p /home/admin-service-operator/workspace
WORKDIR /home/admin-service-operator/workspace
COPY --from=server-test-and-build /home/developer/workspace/target/release/admin_service ./
EXPOSE 3001
CMD [ "./admin_service" ]
