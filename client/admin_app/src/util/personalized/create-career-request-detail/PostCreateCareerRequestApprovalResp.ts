export class PostCreateCareerRequestApprovalResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostCreateCareerRequestApprovalResp {
    return new PostCreateCareerRequestApprovalResp()
  }
}
