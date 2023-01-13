import { ConsultationDateTime } from '../ConsultationDateTime'

export type ConsultantSideConsultation = {
  /* eslint-disable camelcase */
  consultation_id: number,
  user_account_id: number, // 相談申し込み者のユーザーID
  meeting_date_time_in_jst: ConsultationDateTime,
  /* eslint-enable camelcase */
}
