import { UserSideConsultation } from './UserSideConsultation'

export class GetUserSideConsultationResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly userSideConsultation: UserSideConsultation) {}

  public static create (userSideConsultation: UserSideConsultation): GetUserSideConsultationResp {
    return new GetUserSideConsultationResp(userSideConsultation)
  }

  public getConsultationsResult (): UserSideConsultation {
    return this.userSideConsultation
  }
}
