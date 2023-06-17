export class PostResumeSettlementReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostResumeSettlementReqResp {
    return new PostResumeSettlementReqResp()
  }
}
