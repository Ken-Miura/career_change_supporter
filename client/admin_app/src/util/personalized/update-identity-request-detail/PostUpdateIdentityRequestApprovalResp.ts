export class PostUpdateIdentityRequestApprovalResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostUpdateIdentityRequestApprovalResp {
    return new PostUpdateIdentityRequestApprovalResp()
  }
}
