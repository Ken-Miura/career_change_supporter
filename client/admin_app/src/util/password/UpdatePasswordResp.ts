export class UpdatePasswordResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
    private constructor () {}
  public static create (): UpdatePasswordResp {
    return new UpdatePasswordResp()
  }
}
