export class PostCreateIdentityRequestRejectionResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostCreateIdentityRequestRejectionResp {
    return new PostCreateIdentityRequestRejectionResp()
  }
}
