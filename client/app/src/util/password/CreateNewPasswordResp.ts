export class CreateNewPasswordResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): CreateNewPasswordResp {
    return new CreateNewPasswordResp()
  }
}
