export class PostRecoveryCodeResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostRecoveryCodeResp {
    return new PostRecoveryCodeResp()
  }
}
