import { Identity } from '@/util/profile/Identity'
import { createStore } from 'vuex'
import { SET_APPLY_NEW_PASSWORD_RESULT_MESSAGE, SET_IDENTITY } from './mutationTypes'

export type State = {
  applyNewPasswordResultMessage: string | null
  identity: Identity | null
};

// 下記URLにVuexにてTypescriptの型推論を有効にするためにkeyが必要と記載されているが
// このkeyを利用するとjestを用いた単体テストの際、vuexをモック化してもエラーが発生し、テストができないため利用しないようにする
// https://next.vuex.vuejs.org/guide/typescript-support.html#typing-usestore-composition-function
// export const key: InjectionKey<Store<State>> = Symbol('symbol for specifying vuex type')

export default createStore<State>({
  state: {
    applyNewPasswordResultMessage: null,
    identity: null
  },
  mutations: {
    [SET_APPLY_NEW_PASSWORD_RESULT_MESSAGE] (state: State, message: string) {
      state.applyNewPasswordResultMessage = message
    },
    [SET_IDENTITY] (state: State, identity: Identity | null) {
      state.identity = identity
    }
  },
  actions: {
  },
  modules: {
  }
})
