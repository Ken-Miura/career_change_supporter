export class PostRefundFromAwaitingPaymentResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostRefundFromAwaitingPaymentResp {
    return new PostRefundFromAwaitingPaymentResp()
  }
}
