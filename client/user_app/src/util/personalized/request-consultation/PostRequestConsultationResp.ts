export class PostRequestConsultationResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostRequestConsultationResp {
    return new PostRequestConsultationResp()
  }
}
