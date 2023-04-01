import { Identity } from '@/util/personalized/profile/Identity'
import { BankAccount } from '@/util/personalized/BankAccount'
import { createStore } from 'vuex'
import { SET_PASSWORD_UPDATE_RESULT_MESSAGE, SET_BANK_ACCOUNT, SET_FEE_PER_HOUR_IN_YEN, SET_IDENTITY, SET_CONSULTANT_SEARCH_PARAM, SET_PAY_JP, SET_RECOVERY_CODE } from './mutationTypes'
import { ConsultantSearchParam } from '@/util/personalized/ConsultantSearchParam'

export type State = {
  passwordUpdateResultMessage: string | null,
  identity: Identity | null,
  feePerHourInYen: number | null,
  bankAccount: BankAccount | null,
  consultantSearchParam: ConsultantSearchParam | null,
  // PAY.JPから型定義が提供されていないため、anyでの扱いを許容する
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  payJp: any,
  recoveryCode: string | null
};

// 下記URLにVuexにてTypescriptの型推論を有効にするためにkeyが必要と記載されているが
// このkeyを利用するとjestを用いた単体テストの際、vuexをモック化してもエラーが発生し、テストができないため利用しないようにする
// https://next.vuex.vuejs.org/guide/typescript-support.html#typing-usestore-composition-function
// export const key: InjectionKey<Store<State>> = Symbol('symbol for specifying vuex type')

export default createStore<State>({
  state: {
    passwordUpdateResultMessage: null,
    identity: null,
    feePerHourInYen: null,
    bankAccount: null,
    consultantSearchParam: null,
    payJp: null,
    recoveryCode: null
  },
  mutations: {
    [SET_PASSWORD_UPDATE_RESULT_MESSAGE] (state: State, message: string) {
      state.passwordUpdateResultMessage = message
    },
    [SET_IDENTITY] (state: State, identity: Identity | null) {
      state.identity = identity
    },
    [SET_FEE_PER_HOUR_IN_YEN] (state: State, feePerHourInYen: number | null) {
      state.feePerHourInYen = feePerHourInYen
    },
    [SET_BANK_ACCOUNT] (state: State, bankAccount: BankAccount | null) {
      state.bankAccount = bankAccount
    },
    [SET_CONSULTANT_SEARCH_PARAM] (state: State, consultantSearchParam: ConsultantSearchParam | null) {
      state.consultantSearchParam = consultantSearchParam
    },
    // PAY.JPから型定義が提供されていないため、anyでの扱いを許容する
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    [SET_PAY_JP] (state: State, payJp: any) {
      state.payJp = payJp
    },
    [SET_RECOVERY_CODE] (state: State, recoveryCode: string) {
      state.recoveryCode = recoveryCode
    }
  },
  actions: {
  },
  modules: {
  }
})
