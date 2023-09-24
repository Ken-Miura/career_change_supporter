import { ConsultationDateTime } from '../ConsultationDateTime'

export type ConsultationRequest = {
  /* eslint-disable camelcase */
  consultant_id: number,
  fee_per_hour_in_yen: number,
  first_candidate_in_jst: ConsultationDateTime,
  second_candidate_in_jst: ConsultationDateTime,
  third_candidate_in_jst: ConsultationDateTime
  /* eslint-enable camelcase */
}
