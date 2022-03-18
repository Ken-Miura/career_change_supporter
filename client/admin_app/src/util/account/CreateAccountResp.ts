export class CreateAccountResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): CreateAccountResp {
    return new CreateAccountResp()
  }
}
