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
        language-pack-en \
        libssl-dev \
        lldb \
        pkg-config \
        postgresql-client-14 \
        vim && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

ENV LANG="en_US.UTF-8"

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
RUN export SEA_ORM_CLI_VERSION=0.12.2 && \
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

FROM --platform=linux/amd64 ubuntu:22.04 as application-base
RUN apt-get update && \
    export DEBIAN_FRONTEND=noninteractive && \
    apt-get install -y ca-certificates language-pack-en && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*
ENV LANG="en_US.UTF-8"

FROM application-base as user-service
RUN useradd -m -s /bin/bash -U user-service-operator
USER user-service-operator
RUN mkdir -p /home/user-service-operator/workspace
WORKDIR /home/user-service-operator/workspace
COPY --from=server-test-and-build /home/developer/workspace/target/release/user_service ./
EXPOSE 3000
CMD [ "./user_service" ]

FROM application-base as admin-service
RUN useradd -m -s /bin/bash -U admin-service-operator
USER admin-service-operator
RUN mkdir -p /home/admin-service-operator/workspace
WORKDIR /home/admin-service-operator/workspace
COPY --from=server-test-and-build /home/developer/workspace/target/release/admin_service ./
EXPOSE 3001
CMD [ "./admin_service" ]

FROM application-base as admin-account
RUN useradd -m -s /bin/bash -U admin-account-operator
USER admin-account-operator
RUN mkdir -p /home/admin-account-operator/workspace
WORKDIR /home/admin-account-operator/workspace
COPY --from=server-test-and-build /home/developer/workspace/target/release/admin_account ./
ENV PATH="${PATH}:/home/admin-account-operator/workspace"
ENTRYPOINT [ "admin_account" ]

FROM application-base as batch-processor-base
RUN useradd -m -s /bin/bash -U batch-processor
USER batch-processor
RUN mkdir -p /home/batch-processor/workspace
WORKDIR /home/batch-processor/workspace
ENV PATH="${PATH}:/home/batch-processor/workspace"

FROM batch-processor-base as delete-expired-consultation-reqs
COPY --from=server-test-and-build /home/developer/workspace/target/release/delete_expired_consultation_reqs ./
ENTRYPOINT [ "delete_expired_consultation_reqs" ]

FROM batch-processor-base as delete-expired-deleted-user-accounts
COPY --from=server-test-and-build /home/developer/workspace/target/release/delete_expired_deleted_user_accounts ./
ENTRYPOINT [ "delete_expired_deleted_user_accounts" ]

FROM batch-processor-base as delete-expired-pwd-change-reqs
COPY --from=server-test-and-build /home/developer/workspace/target/release/delete_expired_pwd_change_reqs ./
ENTRYPOINT [ "delete_expired_pwd_change_reqs" ]

FROM batch-processor-base as delete-expired-stopped-settlements
COPY --from=server-test-and-build /home/developer/workspace/target/release/delete_expired_stopped_settlements ./
ENTRYPOINT [ "delete_expired_stopped_settlements" ]

FROM batch-processor-base as delete-expired-temp-accounts
COPY --from=server-test-and-build /home/developer/workspace/target/release/delete_expired_temp_accounts ./
ENTRYPOINT [ "delete_expired_temp_accounts" ]

FROM batch-processor-base as delete-expired-temp-mfa-secrets
COPY --from=server-test-and-build /home/developer/workspace/target/release/delete_expired_temp_mfa_secrets ./
ENTRYPOINT [ "delete_expired_temp_mfa_secrets" ]

FROM batch-processor-base as make-payment-of-unhandled-settlement
COPY --from=server-test-and-build /home/developer/workspace/target/release/make_payment_of_unhandled_settlement ./
ENTRYPOINT [ "make_payment_of_unhandled_settlement" ]

# DB初期化以外のpsql操作が必要になった場合は、このイメージを利用し、
# 利用頻度により随時必要なイメージとして切り出すか検討する
FROM --platform=linux/amd64 alpine:3.18.2 as psql-client
RUN apk add --update bash postgresql14-client && \
        rm -rf /var/cache/apk/*
CMD [ "psql", "--help" ]

FROM psql-client as db-initializer
# ビルドコンテキストはserverディレクトリ直下であると仮定
COPY data_store_setup_files/initdb/init.sh .
CMD [ "./init.sh" ]

FROM --platform=linux/amd64 rust:1.70.0-alpine as migration-tool
RUN apk add --update build-base pkgconfig openssl-dev && \
        rm -rf /var/cache/apk/*
WORKDIR /workspace
RUN export SEA_ORM_CLI_VERSION=0.12.2 && \
    cargo install --locked --version ${SEA_ORM_CLI_VERSION} sea-orm-cli
# ビルドコンテキストはserverディレクトリ直下であると仮定
COPY . .
ENTRYPOINT [ "sea-orm-cli" ]

# インデックス初期化以外のcurl操作が必要になった場合は https://hub.docker.com/r/alpine/curl/tags を利用し、
# 利用頻度により随時必要なイメージとして切り出すか検討する
FROM --platform=linux/amd64 alpine/curl:8.1.2 as index-initializer
# ビルドコンテキストはserverディレクトリ直下であると仮定
COPY data_store_setup_files/opensearch/index_definition/index.json .
CMD [ "sh", "-c", "curl -XPUT -H \"Content-Type: application/json\" --data @index.json -u ${USER_NAME}:${PASSWORD} ${ENDPOINT_URI}/users" ]
