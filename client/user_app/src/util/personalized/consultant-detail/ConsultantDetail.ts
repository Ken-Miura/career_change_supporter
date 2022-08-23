import { ConsultantCareerDetail } from './ConsultantCareerDetail'

export type ConsultantDetail = {
  /* eslint-disable camelcase */
  consultant_id: number,
  fee_per_hour_in_yen: number,
  rating: number | null,
  num_of_rated: number,
  careers: ConsultantCareerDetail[],
  /* eslint-enable camelcase */
}
