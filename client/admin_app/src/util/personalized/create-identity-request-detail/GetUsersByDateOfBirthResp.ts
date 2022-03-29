import { User } from './User'

export class GetUsersByDateOfBirthResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly users: User[]) {}
  public static create (users: User[]): GetUsersByDateOfBirthResp {
    return new GetUsersByDateOfBirthResp(users)
  }

  public getUsers (): User[] {
    return this.users
  }
}
