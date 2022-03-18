export class PostIdentityResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
    private constructor () {}
  public static create (): PostIdentityResp {
    return new PostIdentityResp()
  }
}
