import { ConsultationRequestDetail } from './ConsultationRequestDetail'

export class GetConsultationRequestDetailResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly detail: ConsultationRequestDetail) {}

  public static create (detail: ConsultationRequestDetail): GetConsultationRequestDetailResp {
    return new GetConsultationRequestDetailResp(detail)
  }

  public getConsultationRequestDetail (): ConsultationRequestDetail {
    return this.detail
  }
}
