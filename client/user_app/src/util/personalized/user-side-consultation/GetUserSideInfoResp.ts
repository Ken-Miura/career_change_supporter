import { UserSideInfo } from './UserSideInfo'

export class GetUserSideInfoResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly userSideInfo: UserSideInfo) {}

  public static create (userSideInfo: UserSideInfo): GetUserSideInfoResp {
    return new GetUserSideInfoResp(userSideInfo)
  }

  public getUserSideInfo (): UserSideInfo {
    return this.userSideInfo
  }
}
