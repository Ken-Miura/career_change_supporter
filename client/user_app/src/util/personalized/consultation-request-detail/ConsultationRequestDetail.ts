import { ConsultationDateTime } from '../ConsultationDateTime'

export type ConsultationRequestDetail = {
  /* eslint-disable camelcase */
  consultation_req_id: number,
  user_account_id: number,
  user_rating: string | null,
  num_of_rated_of_user: number,
  fee_per_hour_in_yen: number,
  first_candidate_in_jst: ConsultationDateTime,
  second_candidate_in_jst: ConsultationDateTime,
  third_candidate_in_jst: ConsultationDateTime,
  /* eslint-enable camelcase */
}
