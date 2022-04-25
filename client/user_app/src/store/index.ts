import { Identity } from '@/util/personalized/profile/Identity'
import { BankAccount } from '@/util/personalized/reward/BankAccount'
import { createStore } from 'vuex'
import { SET_PASSWORD_UPDATE_RESULT_MESSAGE, SET_BANK_ACCOUNT, SET_FEE_PER_HOUR_IN_YEN, SET_IDENTITY, SET_POST_IDENTITY_RESULT_MESSAGE } from './mutationTypes'

export type State = {
  postIdentityResultMessage: string | null,
  passwordUpdateResultMessage: string | null,
  identity: Identity | null,
  feePerHourInYen: number | null,
  bankAccount: BankAccount | null
};

// 下記URLにVuexにてTypescriptの型推論を有効にするためにkeyが必要と記載されているが
// このkeyを利用するとjestを用いた単体テストの際、vuexをモック化してもエラーが発生し、テストができないため利用しないようにする
// https://next.vuex.vuejs.org/guide/typescript-support.html#typing-usestore-composition-function
// export const key: InjectionKey<Store<State>> = Symbol('symbol for specifying vuex type')

export default createStore<State>({
  state: {
    postIdentityResultMessage: null,
    passwordUpdateResultMessage: null,
    identity: null,
    feePerHourInYen: null,
    bankAccount: null
  },
  mutations: {
    [SET_POST_IDENTITY_RESULT_MESSAGE] (state: State, message: string) {
      state.postIdentityResultMessage = message
    },
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
    }
  },
  actions: {
  },
  modules: {
  }
})
