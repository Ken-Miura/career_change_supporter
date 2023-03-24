import { TempMfaSecret } from './TempMfaSecret'

export class GetTempMfaSecretResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly tempMfaSecret: TempMfaSecret) {}

  public static create (tempMfaSecret: TempMfaSecret): GetTempMfaSecretResp {
    return new GetTempMfaSecretResp(tempMfaSecret)
  }

  public getTempMfaSecret (): TempMfaSecret {
    return this.tempMfaSecret
  }
}
