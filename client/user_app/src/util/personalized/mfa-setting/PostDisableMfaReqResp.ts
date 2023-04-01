export class PostDisableMfaReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostDisableMfaReqResp {
    return new PostDisableMfaReqResp()
  }
}
