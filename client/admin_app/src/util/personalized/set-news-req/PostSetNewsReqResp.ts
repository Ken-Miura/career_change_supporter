export class PostSetNewsReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostSetNewsReqResp {
    return new PostSetNewsReqResp()
  }
}
