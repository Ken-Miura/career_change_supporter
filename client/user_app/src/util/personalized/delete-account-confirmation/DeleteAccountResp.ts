export class DeleteAccountResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): DeleteAccountResp {
    return new DeleteAccountResp()
  }
}
