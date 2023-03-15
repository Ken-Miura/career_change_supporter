export class PostBeginRequestConsultationResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly chargeId: string) {}

  public static create (chargeId: string): PostBeginRequestConsultationResp {
    return new PostBeginRequestConsultationResp(chargeId)
  }

  public getChargeId (): string {
    return this.chargeId
  }
}
