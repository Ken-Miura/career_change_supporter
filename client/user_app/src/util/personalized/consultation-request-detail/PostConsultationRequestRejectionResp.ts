export class PostConsultationRequestRejectionResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostConsultationRequestRejectionResp {
    return new PostConsultationRequestRejectionResp()
  }
}
