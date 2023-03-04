export class PostUserRatingResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostUserRatingResp {
    return new PostUserRatingResp()
  }
}
