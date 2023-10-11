export class PostRefundFromAwaitingWithdrawalResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostRefundFromAwaitingWithdrawalResp {
    return new PostRefundFromAwaitingWithdrawalResp()
  }
}
