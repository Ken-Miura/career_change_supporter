export class LogoutResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): LogoutResp {
    return new LogoutResp()
  }
}
