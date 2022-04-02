export class PostCreateIdentityRequestApprovalResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostCreateIdentityRequestApprovalResp {
    return new PostCreateIdentityRequestApprovalResp()
  }
}
