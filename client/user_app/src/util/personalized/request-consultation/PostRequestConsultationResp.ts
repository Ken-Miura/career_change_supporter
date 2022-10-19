export class PostRequestConsultationResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly chargeId: string) {}

  public static create (chargeId: string): PostRequestConsultationResp {
    return new PostRequestConsultationResp(chargeId)
  }

  public getChargeId (): string {
    return this.chargeId
  }
}
