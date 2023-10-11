export class PostReceiptOfConsultationResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): PostReceiptOfConsultationResp {
    return new PostReceiptOfConsultationResp()
  }
}
