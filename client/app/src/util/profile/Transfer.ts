import { Ymd } from '../Ymd'

export type Transfer = {
    /* eslint-disable camelcase */
    status: 'pending' | 'paid' | 'failed' | 'stop' | 'carried_over' | 'recombination',
    amount: number,
    scheduled_date_in_jst: Ymd,
    // status == 'paid'のときのみ存在
    transfer_amount: number | null,
    transfer_date_in_jst: Ymd | null,
    // status == 'carried_over'のときのみ存在
    carried_balance: number | null,
    /* eslint-enable camelcase */
}
