export class CreatePwdChangeReqResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor () {}
  public static create (): CreatePwdChangeReqResp {
    return new CreatePwdChangeReqResp()
  }
}
