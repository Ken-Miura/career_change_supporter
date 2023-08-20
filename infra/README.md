2023年8月13日現在の状況
テスト用の環境を用意し、最低限の動作確認は完了した。しかし、RDS、OpenSearch等々は中々お金がかかるので一時削除を行う。
削除する前に、テストで動作確認出来た状態に戻せるように設定を保存しておきたかった。従って、既存のリソースをFormer2の機能を試し、テンプレートを作成した。
それをコミットして残しておく。本番用にパラメータ調整した、かつ必要なリソース全体を持っているテンプレートは最後に作成する予定。

テンプレートに保存していない内容
利用用がかかるため、ccs-vpce-ssmという名称のSystem Manager (Paramerter Store) へのエンドポイントを保持していたが削除した。
このエンドポイントは、ECSがParamerter Storeから環境変数を取得するために、publicサブネットに関連付けられていた。