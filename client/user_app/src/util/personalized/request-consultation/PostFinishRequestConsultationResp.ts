export class PostFinishRequestConsultationResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}

  public static create (): PostFinishRequestConsultationResp {
    return new PostFinishRequestConsultationResp()
  }
}
