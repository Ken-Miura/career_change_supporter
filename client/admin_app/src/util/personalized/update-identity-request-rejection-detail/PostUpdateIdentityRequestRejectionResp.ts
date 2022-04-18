export class PostUpdateIdentityRequestRejectionResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostUpdateIdentityRequestRejectionResp {
    return new PostUpdateIdentityRequestRejectionResp()
  }
}
