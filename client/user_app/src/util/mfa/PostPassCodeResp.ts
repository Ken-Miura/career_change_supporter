export class PostPassCodeResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostPassCodeResp {
    return new PostPassCodeResp()
  }
}
