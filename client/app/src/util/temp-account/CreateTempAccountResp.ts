export class CreateTempAccountResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly emailAddress: string) {}
  public static create (emailAddress: string): CreateTempAccountResp {
    return new CreateTempAccountResp(emailAddress)
  }

  public getEmailAddress (): string {
    return this.emailAddress
  }
}
