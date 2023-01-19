import { ConsultantSideInfo } from './ConsultantSideInfo'

export class GetConsultantSideInfoResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly consultantSideInfo: ConsultantSideInfo) {}

  public static create (consultantSideInfo: ConsultantSideInfo): GetConsultantSideInfoResp {
    return new GetConsultantSideInfoResp(consultantSideInfo)
  }

  public getConsultantSideInfo (): ConsultantSideInfo {
    return this.consultantSideInfo
  }
}
