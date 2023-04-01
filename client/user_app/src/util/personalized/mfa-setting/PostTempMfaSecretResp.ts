export class PostTempMfaSecretResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostTempMfaSecretResp {
    return new PostTempMfaSecretResp()
  }
}
