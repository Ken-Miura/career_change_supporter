export class PostAwaitingWithdrawalResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostAwaitingWithdrawalResp {
    return new PostAwaitingWithdrawalResp()
  }
}
