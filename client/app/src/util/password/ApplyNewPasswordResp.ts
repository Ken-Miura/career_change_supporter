export class ApplyNewPasswordResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
    private constructor () {}
  public static create (): ApplyNewPasswordResp {
    return new ApplyNewPasswordResp()
  }
}
