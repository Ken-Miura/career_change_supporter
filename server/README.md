S3バケット作成
```
aws --endpoint-url=http://localhost:4566 s3 mb s3://identification-images --profile=localstack
aws --endpoint-url=http://localhost:4566 s3 mb s3://career-confirmation-images --profile=localstack
```

NOTE:
定期処理で対応する予定の処理
- 期限切れのtemp_accountレコードの削除
