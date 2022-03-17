import { Rewards } from './Rewards'

export class GetRewardsResp {
  // createからアクセスしているため、意味のないコンストラクタではない
  // eslint-disable-next-line
  private constructor (private readonly rewards: Rewards) {}

  public static create (rewards: Rewards): GetRewardsResp {
    return new GetRewardsResp(rewards)
  }

  public getRewards (): Rewards {
    return this.rewards
  }
}
