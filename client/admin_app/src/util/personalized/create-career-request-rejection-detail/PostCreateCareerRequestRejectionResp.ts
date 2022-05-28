export class PostCreateCareerRequestRejectionResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostCreateCareerRequestRejectionResp {
    return new PostCreateCareerRequestRejectionResp()
  }
}
