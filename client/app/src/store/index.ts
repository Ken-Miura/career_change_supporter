import { Identity } from '@/util/profile/Identity'
import { createStore } from 'vuex'

export type State = {
  applyNewPasswordResultMessage: string | null
  identity: Identity | null
};

// 下記URLにVuexにてTypescriptの型推論を有効にするためにkeyが必要と記載されているが
// このkeyを利用するとjestを用いた単体テストの際、vuexをモック化してもエラーが発生し、テストができないため利用しないようにする
// (keyがなくても型推論ができているのでなおさら必要ないように見える)
// https://next.vuex.vuejs.org/guide/typescript-support.html#typing-usestore-composition-function
// export const key: InjectionKey<Store<State>> = Symbol('symbol for specifying vuex type')

export default createStore<State>({
  state: {
    applyNewPasswordResultMessage: null,
    identity: null
  },
  mutations: {
    setApplyNewPasswordResultMessage (state: State, message: string) {
      state.applyNewPasswordResultMessage = message
    },
    setIdentity (state: State, identity: Identity | null) {
      state.identity = identity
    }
  },
  actions: {
  },
  modules: {
  }
})
