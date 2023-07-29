FROM node:18.17.0

WORKDIR /workspace
RUN npm install aws-ses-v2-local@2.2.0
# default port is 8005
EXPOSE 8005
# ホストはデフォルトでlocalhostとなるため、コンテナ外部からアクセスできない
# コンテナ外部からもパケットを受け取れるように--hostオプションに0.0.0.0を指定する
CMD [ "npx", "aws-ses-v2-local", "--host", "0.0.0.0" ]
