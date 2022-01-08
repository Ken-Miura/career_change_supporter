import { Store, useStore as baseUseStore } from 'vuex'
import { key, State } from '.'

export function useStore (): Store<State> {
  return baseUseStore(key)
}
