import { ConsultationDateTime } from '../ConsultationDateTime'

export type UnratedUser = {
  /* eslint-disable camelcase */
  consultation_id: number,
  user_account_id: number,
  meeting_date_time_in_jst: ConsultationDateTime,
  /* eslint-enable camelcase */
}
