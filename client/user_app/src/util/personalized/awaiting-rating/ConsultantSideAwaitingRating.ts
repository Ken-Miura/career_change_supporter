import { ConsultationDateTime } from '../ConsultationDateTime'

export type ConsultantSideAwaitingRating = {
  /* eslint-disable camelcase */
  consultant_rating_id: number,
  user_account_id: number, // 相談申し込み者のユーザーID
  meeting_date_time_in_jst: ConsultationDateTime,
  /* eslint-enable camelcase */
}
