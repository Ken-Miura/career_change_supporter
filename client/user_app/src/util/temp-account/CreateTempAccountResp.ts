export class CreateTempAccountResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): CreateTempAccountResp {
    return new CreateTempAccountResp()
  }
}
