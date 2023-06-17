export class PostRefundReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostRefundReqResp {
    return new PostRefundReqResp()
  }
}
