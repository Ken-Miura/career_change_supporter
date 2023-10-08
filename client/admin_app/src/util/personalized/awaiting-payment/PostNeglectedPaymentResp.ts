export class PostNeglectedPaymentResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostNeglectedPaymentResp {
    return new PostNeglectedPaymentResp()
  }
}
