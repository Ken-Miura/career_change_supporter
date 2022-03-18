export class RefreshResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): RefreshResp {
    return new RefreshResp()
  }
}
