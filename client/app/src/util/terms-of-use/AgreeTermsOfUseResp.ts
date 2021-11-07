export class AgreeTermsOfUseResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
    private constructor () {}
  public static create (): AgreeTermsOfUseResp {
    return new AgreeTermsOfUseResp()
  }
}
