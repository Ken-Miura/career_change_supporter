export class PostEnableMfaReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly recoveryCode: string) {}

  public static create (recoveryCode: string): PostEnableMfaReqResp {
    return new PostEnableMfaReqResp(recoveryCode)
  }

  public getRecoveryCode (): string {
    return this.recoveryCode
  }
}
