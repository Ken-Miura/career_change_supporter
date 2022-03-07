# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.217.4/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="bullseye"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

# [Optional] Add packages to this section to install additional packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
     && apt-get -y install --no-install-recommends dnsutils tcpdump

# cargoを用いて開発に有用なツールをインストールする。
# cargoを実行した際、registry以下にcrateがダウンロードされ、ディレクトリとファイルが生成される。
# USER切り替えをしないと、それらがrootとして生成されるので、開発時に実際に利用するユーザーのvscodeに切り替え
USER vscode
RUN cargo install --locked ripgrep \
    && cargo install --locked ripgrep_all \
    && cargo install --locked fd-find \
    && cargo install --locked bat \
    && cargo install --locked git-delta

# バックエンドの開発に必要なツールをインストール
RUN cargo install --locked sea-orm-cli
