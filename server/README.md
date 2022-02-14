ローカルの開発環境をセットアップするために、下記の操作を一度実施する必要がある。

DBセットアップ
```
cd common
diesel setup --database-url=postgres://postgres:example@db/ccs_db
```

.envファイル例
```
DB_URL_FOR_USER_APP=postgres://user_app:test1234@db/ccs_db
SOCKET_FOR_USER_APP=0.0.0.0:3000
SOCKET_FOR_SMTP_SERVER=smtp:1025
URL_FOR_FRONT_END=https://localhost:8080
URL_FOR_REDIS_SERVER=redis://cache:6379
TERMS_OF_USE_VERSION=1
PAYMENT_PLATFORM_API_URL=https://api.pay.jp
PAYMENT_PLATFORM_API_USERNAME=${your_username}
PAYMENT_PLATFORM_API_PASSWORD=${your_password}
```
