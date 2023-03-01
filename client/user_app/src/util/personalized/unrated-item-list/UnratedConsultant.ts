import { ConsultationDateTime } from '../ConsultationDateTime'

export type UnratedConsultant = {
  /* eslint-disable camelcase */
  consultant_rating_id: number,
  consultant_id: number,
  meeting_date_time_in_jst: ConsultationDateTime,
  /* eslint-enable camelcase */
}
