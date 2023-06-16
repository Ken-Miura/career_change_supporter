export class PostStopSettlementReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostStopSettlementReqResp {
    return new PostStopSettlementReqResp()
  }
}
