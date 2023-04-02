import { LoginResult } from './LoginResult'

export class LoginResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly ls: LoginResult) {}

  public static create (ls: LoginResult): LoginResp {
    return new LoginResp(ls)
  }

  public getLoginResult (): LoginResult {
    return this.ls
  }
}
