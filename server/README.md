S3バケット作成
```
aws --endpoint-url=http://localhost:4566 s3 mb s3://identification-images --profile=localstack
aws --endpoint-url=http://localhost:4566 s3 mb s3://career-confirmation-images --profile=localstack
```

.envファイル例
```
DB_URL_FOR_USER_APP=postgres://user_app:test1234@localhost/ccs_db
SOCKET_FOR_USER_APP=127.0.0.1:3000
SOCKET_FOR_SMTP_SERVER=127.0.0.1:1025
URL_FOR_FRONT_END=https://localhost:8080
URL_FOR_REDIS_SERVER=redis://127.0.0.1:6379
TERMS_OF_USE_VERSION=1
PAYMENT_PLATFORM_API_URL=https://api.pay.jp
PAYMENT_PLATFORM_API_USERNAME=${your_username}
PAYMENT_PLATFORM_API_PASSWORD=${your_password}
```

TODO:
定期処理で対応する予定の処理
- 期限切れのtemp_accountレコードの削除
- 期限切れのnew_passwordレコードの削除
