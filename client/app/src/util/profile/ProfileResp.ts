export class GetProfileResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly emailAddress: string) {}

  public static create (emailAddress: string): GetProfileResp {
    return new GetProfileResp(emailAddress)
  }

  public getEmailAddress (): string {
    return this.emailAddress
  }
}
