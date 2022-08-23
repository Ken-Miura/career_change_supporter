import { ConsultantDetail } from './ConsultantDetail'

export class GetConsultantDetailResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly consultantDetail: ConsultantDetail) {}

  public static create (consultantDetail: ConsultantDetail): GetConsultantDetailResp {
    return new GetConsultantDetailResp(consultantDetail)
  }

  public getConsultantDetail (): ConsultantDetail {
    return this.consultantDetail
  }
}
