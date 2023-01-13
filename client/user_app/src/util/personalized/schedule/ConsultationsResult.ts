import { ConsultantSideConsultation } from './ConsultantSideConsultation'
import { UserSideConsultation } from './UserSideConsultation'

export type ConsultationsResult = {
  /* eslint-disable camelcase */
  user_side_consultations: UserSideConsultation[],
  consultant_side_consultations: ConsultantSideConsultation[],
  /* eslint-enable camelcase */
}
