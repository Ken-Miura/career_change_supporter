export class GetFeePerHourInYenForApplicationResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly feePerHourInYen: number) {}

  public static create (feePerHourInYen: number): GetFeePerHourInYenForApplicationResp {
    return new GetFeePerHourInYenForApplicationResp(feePerHourInYen)
  }

  public getFeePerHourInYenForApplication (): number {
    return this.feePerHourInYen
  }
}
