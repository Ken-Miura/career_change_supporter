import { Identity } from './Identity'

export class GetIdentityByUserAccountIdResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly identity: Identity) {}
  public static create (identity: Identity): GetIdentityByUserAccountIdResp {
    return new GetIdentityByUserAccountIdResp(identity)
  }

  public getIdentity (): Identity {
    return this.identity
  }
}
