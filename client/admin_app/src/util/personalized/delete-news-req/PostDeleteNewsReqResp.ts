export class PostDeleteNewsReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostDeleteNewsReqResp {
    return new PostDeleteNewsReqResp()
  }
}
