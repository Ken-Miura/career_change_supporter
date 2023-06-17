export class PostMakePaymentReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostMakePaymentReqResp {
    return new PostMakePaymentReqResp()
  }
}
