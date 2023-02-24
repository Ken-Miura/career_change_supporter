import { ConsultationDateTime } from '../ConsultationDateTime'

export type UserSideAwaitingRating = {
  /* eslint-disable camelcase */
  user_rating_id: number,
  consultant_id: number, // 相談相手のユーザーID
  meeting_date_time_in_jst: ConsultationDateTime,
  /* eslint-enable camelcase */
}
