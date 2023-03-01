import { UnratedConsultant } from './UnratedConsultant'
import { UnratedUser } from './UnratedUser'

export type UnratedItemsResult = {
  /* eslint-disable camelcase */
  unrated_consultants: UnratedConsultant[],
  unrated_users: UnratedUser[],
  /* eslint-enable camelcase */
}
