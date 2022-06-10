export class PostFeePerHourInYenResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostFeePerHourInYenResp {
    return new PostFeePerHourInYenResp()
  }
}
