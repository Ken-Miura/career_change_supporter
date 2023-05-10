import { UserAccountSearchParam } from '@/util/personalized/user-account-search/UserAccountSearchParam'
import { createStore } from 'vuex'
import { SET_USER_ACCOUNT_SEARCH_PARAM } from './mutationTypes'

export type State = {
  userAccountSearchParam: UserAccountSearchParam | null
};

// 下記URLにVuexにてTypescriptの型推論を有効にするためにkeyが必要と記載されているが
// このkeyを利用するとjestを用いた単体テストの際、vuexをモック化してもエラーが発生し、テストができないため利用しないようにする
// https://next.vuex.vuejs.org/guide/typescript-support.html#typing-usestore-composition-function
// export const key: InjectionKey<Store<State>> = Symbol('symbol for specifying vuex type')

export default createStore<State>({
  state: {
    userAccountSearchParam: null
  },
  mutations: {
    [SET_USER_ACCOUNT_SEARCH_PARAM] (state: State, userAccountSearchParam: UserAccountSearchParam) {
      state.userAccountSearchParam = userAccountSearchParam
    }
  },
  actions: {
  },
  modules: {
  }
})
