export class PostCareerResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostCareerResp {
    return new PostCareerResp()
  }
}
