export class PostBankAccountResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostBankAccountResp {
    return new PostBankAccountResp()
  }
}
