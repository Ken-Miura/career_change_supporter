import { Career } from '../Career'

export class GetCareerResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly career: Career) {}
  public static create (career: Career): GetCareerResp {
    return new GetCareerResp(career)
  }

  public getCareer (): Career {
    return this.career
  }
}
