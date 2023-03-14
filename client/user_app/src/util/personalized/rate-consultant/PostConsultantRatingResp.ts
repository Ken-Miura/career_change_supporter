export class PostConsultantRatingResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostConsultantRatingResp {
    return new PostConsultantRatingResp()
  }
}
