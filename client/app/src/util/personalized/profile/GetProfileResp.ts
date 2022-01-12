import { Profile } from './Profile'

export class GetProfileResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly profile: Profile) {}

  public static create (profile: Profile): GetProfileResp {
    return new GetProfileResp(profile)
  }

  public getProfile (): Profile {
    return this.profile
  }
}
