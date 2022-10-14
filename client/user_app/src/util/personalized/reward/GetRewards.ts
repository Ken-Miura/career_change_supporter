import { ApiErrorResp, ApiError } from '../../ApiError'
import { GetRewardsResp } from './GetRewardsResp'
import { Rewards as RawRewards } from './raw-response/Rewards'
import { Rewards } from './Rewards'
import { Transfer } from './Transfer'

export async function getRewards (): Promise<GetRewardsResp | ApiErrorResp> {
  const response = await fetch('/api/rewards', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const rawResult = await response.json() as RawRewards
  const result = convertResult(rawResult)
  return GetRewardsResp.create(result)
}

// Rewardsのlatest_two_transfersの要素内に一意に識別できる値は含まれていない。
// v-forで回す際に一意な識別子が必要になるので、返ってきたレスポンスを画面に表示する前に一意な識別子を生成して含めておく
function convertResult (rawRewards: RawRewards): Rewards {
  const latestTwoTransfers = [] as Transfer[]
  for (let i = 0; i < rawRewards.latest_two_transfers.length; i++) {
    latestTwoTransfers.push({
      transfer_id: i,
      status: rawRewards.latest_two_transfers[i].status,
      amount: rawRewards.latest_two_transfers[i].amount,
      scheduled_date_in_jst: rawRewards.latest_two_transfers[i].scheduled_date_in_jst,
      transfer_amount: rawRewards.latest_two_transfers[i].transfer_amount,
      transfer_date_in_jst: rawRewards.latest_two_transfers[i].transfer_date_in_jst,
      carried_balance: rawRewards.latest_two_transfers[i].carried_balance
    } as Transfer)
  }
  return {
    bank_account: rawRewards.bank_account,
    rewards_of_the_month: rawRewards.rewards_of_the_month,
    rewards_of_the_year: rawRewards.rewards_of_the_year,
    latest_two_transfers: latestTwoTransfers
  } as Rewards
}
