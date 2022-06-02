export class DeleteCareerResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): DeleteCareerResp {
    return new DeleteCareerResp()
  }
}
