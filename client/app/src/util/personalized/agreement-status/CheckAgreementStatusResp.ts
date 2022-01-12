export class CheckAgreementStatusResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
      private constructor () {}
  public static create (): CheckAgreementStatusResp {
    return new CheckAgreementStatusResp()
  }
}
