import { SkyWayCredential } from '../SkyWayCredential'

export type UserSideConsultation = {
  /* eslint-disable camelcase */
  user_account_peer_id: string,
  credential: SkyWayCredential,
  consultant_peer_id: string | null,
  /* eslint-enable camelcase */
}