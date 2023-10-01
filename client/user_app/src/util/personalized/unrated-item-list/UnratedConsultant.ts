import { ConsultationDateTime } from '../ConsultationDateTime'

export type UnratedConsultant = {
  /* eslint-disable camelcase */
  consultation_id: number,
  consultant_id: number,
  meeting_date_time_in_jst: ConsultationDateTime,
  /* eslint-enable camelcase */
}
