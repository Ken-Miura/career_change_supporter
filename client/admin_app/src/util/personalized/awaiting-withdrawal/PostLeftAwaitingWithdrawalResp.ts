export class PostLeftAwaitingWithdrawalResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostLeftAwaitingWithdrawalResp {
    return new PostLeftAwaitingWithdrawalResp()
  }
}
