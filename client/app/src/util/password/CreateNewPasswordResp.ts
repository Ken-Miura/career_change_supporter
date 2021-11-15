export class CreateNewPasswordResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
    private constructor (private readonly emailAddress: string) {}
  public static create (emailAddress: string): CreateNewPasswordResp {
    return new CreateNewPasswordResp(emailAddress)
  }

  public getEmailAddress (): string {
    return this.emailAddress
  }
}
