export class PostConsultationRequestAcceptanceResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostConsultationRequestAcceptanceResp {
    return new PostConsultationRequestAcceptanceResp()
  }
}
