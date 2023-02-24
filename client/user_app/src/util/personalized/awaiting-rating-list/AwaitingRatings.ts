import { ConsultantSideAwaitingRating } from './ConsultantSideAwaitingRating'
import { UserSideAwaitingRating } from './UserSideAwaitingRating'

export type AwaitingRatings = {
  /* eslint-disable camelcase */
  user_side_awaiting_ratings: UserSideAwaitingRating[],
  consultant_side_awaiting_ratings: ConsultantSideAwaitingRating[],
  /* eslint-enable camelcase */
}
