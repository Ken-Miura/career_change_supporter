import { ConsultationDateTime } from '../ConsultationDateTime'

export type UserSideConsultation = {
  /* eslint-disable camelcase */
  consultation_id: number,
  consultant_id: number, // 相談相手のユーザーID
  meeting_date_time_in_jst: ConsultationDateTime,
  /* eslint-enable camelcase */
}
