# CI/CDにはGitHub Actionsを想定している。
# GitHub Actionsで利用可能なマシンのCPUアーキテクチャはx86_64 (amd64) のみ
#   参考: https://docs.github.com/ja/actions/using-github-hosted-runners/about-github-hosted-runners#supported-runners-and-hardware-resources
# 異なるCPUアーキテクチャ向けのイメージをビルドしようとするとかなりの時間を要することになるため、CI/CDでのビルド環境のCPUアーキテクチャに合わせる
# ビルド後のイメージを実際に走らせる環境（AWS ECS）のマシンのCPUアーキテクチャは、デフォルトでLinux、x86_64のためCI/CD環境でビルドしたものがそのまま動作する
#   参考: https://docs.aws.amazon.com/ja_jp/AmazonECS/latest/developerguide/task_definition_parameters.html#runtime-platform
FROM --platform=linux/amd64 ubuntu:22.04 as development

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
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${RUST_VERSION}
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

FROM --platform=linux/amd64 ubuntu:22.04 as user-service
RUN useradd -m -s /bin/bash -U user-service-operator
USER user-service-operator
RUN mkdir -p /home/user-service-operator/workspace
WORKDIR /home/user-service-operator/workspace
COPY --from=server-test-and-build /home/developer/workspace/target/release/user_service ./
EXPOSE 3000
CMD [ "./user_service" ]

FROM --platform=linux/amd64 ubuntu:22.04 as admin-service
RUN useradd -m -s /bin/bash -U admin-service-operator
USER admin-service-operator
RUN mkdir -p /home/admin-service-operator/workspace
WORKDIR /home/admin-service-operator/workspace
COPY --from=server-test-and-build /home/developer/workspace/target/release/admin_service ./
EXPOSE 3001
CMD [ "./admin_service" ]

FROM --platform=linux/amd64 ubuntu:22.04 as admin-account
RUN useradd -m -s /bin/bash -U admin-account-operator
USER admin-account-operator
RUN mkdir -p /home/admin-account-operator/workspace
WORKDIR /home/admin-account-operator/workspace
COPY --from=server-test-and-build /home/developer/workspace/target/release/admin_account ./
CMD [ "./admin_account" ]

# DB初期化以外のpsql操作が必要になった場合は、このイメージを利用し、
# 利用頻度により随時必要なイメージとして切り出すか検討する
FROM --platform=linux/amd64 alpine:3.18.2 as psql-client
RUN apk add --update bash postgresql14-client && \
        rm -rf /var/cache/apk/*
CMD [ "psql", "--help" ]

FROM psql-client as db-initializer
# ビルドコンテキストはserverディレクトリ直下であると仮定
COPY files_for_compose/initdb/init.sh .
CMD [ "./init.sh" ]

FROM --platform=linux/amd64 ubuntu:22.04 as migration-tool
RUN useradd -m -s /bin/bash -U migration-tool-operator
USER migration-tool-operator
RUN mkdir -p /home/migration-tool-operator/workspace
WORKDIR /home/migration-tool-operator/workspace
COPY --from=server-test-and-build /home/developer/.cargo/bin/sea-orm-cli ./
# ビルドコンテキストはserverディレクトリ直下であると仮定
COPY . .
CMD [ "./sea-orm-cli" ]

# インデックス初期化以外のcurl操作が必要になった場合は https://hub.docker.com/r/alpine/curl/tags を利用し、
# 利用頻度により随時必要なイメージとして切り出すか検討する
FROM --platform=linux/amd64 alpine/curl:8.1.2 as index-initializer
# ビルドコンテキストはserverディレクトリ直下であると仮定
COPY files_for_compose/opensearch/index_definition/index.json .
CMD [ "sh", "-c", "curl -XPUT -H \"Content-Type: application/json\" --data @index.json ${SCHEME}://${HOST}:${PORT}/users" ]
