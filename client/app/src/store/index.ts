import { Career } from '@/util/personalized/profile/Career'
import { Identity } from '@/util/personalized/profile/Identity'
import { BankAccount } from '@/util/personalized/reward/BankAccount'
import { createStore } from 'vuex'
import { SET_APPLY_NEW_PASSWORD_RESULT_MESSAGE, SET_BANK_ACCOUNT, SET_CAREERS, SET_FEE_PER_HOUR_IN_YEN, SET_IDENTITY } from './mutationTypes'

export type State = {
  applyNewPasswordResultMessage: string | null,
  identity: Identity | null,
  careers: Career[],
  feePerHourInYen: number | null,
  bankAccount: BankAccount | null
};

// 下記URLにVuexにてTypescriptの型推論を有効にするためにkeyが必要と記載されているが
// このkeyを利用するとjestを用いた単体テストの際、vuexをモック化してもエラーが発生し、テストができないため利用しないようにする
// https://next.vuex.vuejs.org/guide/typescript-support.html#typing-usestore-composition-function
// export const key: InjectionKey<Store<State>> = Symbol('symbol for specifying vuex type')

export default createStore<State>({
  state: {
    applyNewPasswordResultMessage: null,
    identity: null,
    careers: [],
    feePerHourInYen: null,
    bankAccount: null
  },
  mutations: {
    [SET_APPLY_NEW_PASSWORD_RESULT_MESSAGE] (state: State, message: string) {
      state.applyNewPasswordResultMessage = message
    },
    [SET_IDENTITY] (state: State, identity: Identity | null) {
      state.identity = identity
    },
    [SET_CAREERS] (state: State, careers: Career[]) {
      state.careers = careers
    },
    [SET_FEE_PER_HOUR_IN_YEN] (state: State, feePerHourInYen: number | null) {
      state.feePerHourInYen = feePerHourInYen
    },
    [SET_BANK_ACCOUNT] (state: State, bankAccount: BankAccount | null) {
      state.bankAccount = bankAccount
    }
  },
  actions: {
  },
  modules: {
  }
})
