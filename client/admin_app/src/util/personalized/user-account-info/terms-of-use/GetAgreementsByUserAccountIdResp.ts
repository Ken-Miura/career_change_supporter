import { AgreementsResult } from './AgreementsResult'

export class GetAgreementsByUserAccountIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly agreementsResult: AgreementsResult) {}

  public static create (agreementsResult: AgreementsResult): GetAgreementsByUserAccountIdResp {
    return new GetAgreementsByUserAccountIdResp(agreementsResult)
  }

  public getAgreementsResult (): AgreementsResult {
    return this.agreementsResult
  }
}
