<template>
  <div v-on:click="switchNavContentState" class="leading-normal tracking-normal text-white bg-gradient-to-r from-gray-500 to-gray-900" style="font-family: 'Source Sans Pro', sans-serif;">
    <!--Nav-->
    <nav class="fixed w-full z-30 top-0 text-white bg-gradient-to-r from-gray-500 to-gray-900">
      <div class="w-full container mx-auto flex flex-wrap items-center justify-between mt-0 py-2">
        <div class="pl-4 flex items-center">
          <p class="toggleColour text-white no-underline hover:no-underline font-bold text-lg lg:text-4xl">
            就職先・転職先を見極めるためのサイト
          </p>
        </div>
        <div class="block lg:hidden pr-4">
          <button ref="navToggle" class="flex items-center p-1 text-white focus:outline-none focus:shadow-outline transform transition hover:scale-105 duration-300 ease-in-out">
            <svg class="fill-current h-6 w-6" viewBox="0 0 20 20" xmlns="http://www.w3.org/2000/svg">
              <title>Menu</title>
              <path d="M0 3h20v2H0V3zm0 6h20v2H0V9zm0 6h20v2H0v-2z" />
            </svg>
          </button>
        </div>
        <div v-bind:class="['w-full', 'flex-grow', 'lg:flex', 'lg:items-center', 'lg:w-auto', { 'hidden': isHidden }, 'mt-2', 'lg:mt-0', 'bg-gray-500', 'lg:bg-transparent', 'text-black', 'p-4', 'lg:p-0 z-20']">
          <ul class="list-reset lg:flex justify-end flex-1 items-center">
            <li class="mr-3">
              <router-link class="inline-block text-white no-underline py-2 px-4" to="/news">お知らせ</router-link>
            </li>
            <li class="mr-3">
              <router-link class="inline-block text-white no-underline py-2 px-4" to="/new-account">新規登録</router-link>
            </li>
            <li class="mr-3">
              <router-link class="inline-block text-white no-underline py-2 px-4" to="/login">ログイン</router-link>
            </li>
          </ul>
        </div>
      </div>
      <hr class="border-b border-gray-100 opacity-25 my-0 py-0" />
    </nav>
    <!--Hero-->
    <div class="pt-12 lg:pt-24">
      <div class="container px-3 mx-auto flex flex-wrap flex-col lg:flex-row items-center">
        <!--Left Col-->
        <div class="flex flex-col w-full lg:w-2/5 justify-center items-start text-left">
          <h1 class="my-4 text-lg lg:text-3xl font-bold leading-tight">
            就職先・転職先を選ぶ手助けに
          </h1>
          <p class="leading-normal text-base lg:text-xl mb-4 lg:mb-8">
            いざ働いてみると、求人票に書かれていたことと違う、職場の雰囲気が想像していたものと違うなど、働いてみて初めてわかる問題に遭遇したことはありませんか？このサイトは、そのような問題を避けて、就職、転職後のミスマッチを減らす支援をする目的で開設されました。
          </p>
          <h1 class="my-4 text-lg lg:text-3xl font-bold leading-tight">
            匿名でのやり取り
          </h1>
          <p class="leading-normal text-base lg:text-xl mb-8">
            働いてみて初めてわかる問題は、実際に働いている方（働いたことのある方）に事前に相談することで回避できる可能性があります。そのため、このサイトでは、就職先、転職先について確認がしたいユーザーと、そのユーザーが確認したい就職先、転職先で働いている（働いたことがある）ユーザーをマッチングさせ、相談の場を提供することで問題の回避を支援します。マッチングから相談まで、やり取りは匿名（※1）（※2）で行うことが出来、お互いに素性を知られることなく、安心して相談が可能です。
          </p>
          <p class="leading-normal text-base lg:text-xl mb-4 lg:mb-8">
            （※1）相談をしたいユーザーの相談申し込みの判断に利用されるため、相談を受けるユーザーの職務経歴は公開されます。
          </p>
          <p class="leading-normal text-base lg:text-xl mb-8">
            （※2）相談は音声のやり取りで行います。そのため、相談を行う際はマイクとスピーカーが必要となります（映像は利用しないため、カメラは不要です）相談の際の音声は加工されてから相手に送られます。
          </p>
        </div>
        <!--Right Col-->
        <div class="flex justify-center items-center w-full lg:w-3/5 px-2 py-6 text-center">
          <img class="w-full lg:w-4/5" src="consultation.png" />
        </div>
      </div>
    </div>
    <div class="relative -mt-12 lg:-mt-24">
      <svg viewBox="0 0 1428 174" version="1.1" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink">
        <g stroke="none" stroke-width="1" fill="none" fill-rule="evenodd">
          <g transform="translate(-2.000000, 44.000000)" fill="#FFFFFF" fill-rule="nonzero">
            <path d="M0,0 C90.7283404,0.927527913 147.912752,27.187927 291.910178,59.9119003 C387.908462,81.7278826 543.605069,89.334785 759,82.7326078 C469.336065,156.254352 216.336065,153.6679 0,74.9732496" opacity="0.100000001"></path>
            <path d="M100,104.708498 C277.413333,72.2345949 426.147877,52.5246657 546.203633,45.5787101 C666.259389,38.6327546 810.524845,41.7979068 979,55.0741668 C931.069965,56.122511 810.303266,74.8455141 616.699903,111.243176 C423.096539,147.640838 250.863238,145.462612 100,104.708498 Z" opacity="0.100000001"></path>
            <path d="M1046,51.6521276 C1130.83045,29.328812 1279.08318,17.607883 1439,40.1656806 L1439,120 C1271.17211,77.9435312 1140.17211,55.1609071 1046,51.6521276 Z" opacity="0.200000003"></path>
          </g>
          <g transform="translate(-4.000000, 76.000000)" fill="#FFFFFF" fill-rule="nonzero">
            <path d="M0.457,34.035 C57.086,53.198 98.208,65.809 123.822,71.865 C181.454,85.495 234.295,90.29 272.033,93.459 C311.355,96.759 396.635,95.801 461.025,91.663 C486.76,90.01 518.727,86.372 556.926,80.752 C595.747,74.596 622.372,70.008 636.799,66.991 C663.913,61.324 712.501,49.503 727.605,46.128 C780.47,34.317 818.839,22.532 856.324,15.904 C922.689,4.169 955.676,2.522 1011.185,0.432 C1060.705,1.477 1097.39,3.129 1121.236,5.387 C1161.703,9.219 1208.621,17.821 1235.4,22.304 C1285.855,30.748 1354.351,47.432 1440.886,72.354 L1441.191,104.352 L1.121,104.031 L0.457,34.035 Z"></path>
          </g>
        </g>
      </svg>
    </div>
    <!--Footer-->
    <footer class="bg-white">
      <div class="container text-center mx-auto px-8">
        <div class="w-full flex flex-col lg:flex-row py-6">
          <div class="flex-1">
            <ul class="list-reset mb-6">
              <li class="mt-2 inline-block mr-2 lg:block lg:mr-0">
                <router-link class="no-underline hover:underline text-gray-800 hover:text-gray-500" to="/published-terms-of-use">利用規約</router-link>
              </li>
              <li class="mt-2 inline-block mr-2 lg:block lg:mr-0">
                <router-link class="no-underline hover:underline text-gray-800 hover:text-gray-500" to="/transaction-law">特定商取引法に基づく表記</router-link>
              </li>
              <li class="mt-2 inline-block mr-2 lg:block lg:mr-0">
                <router-link class="no-underline hover:underline text-gray-800 hover:text-gray-500" to="/privacy-policy">プライバシーポリシー</router-link>
              </li>
              <li class="mt-2 inline-block mr-2 lg:block lg:mr-0">
                <router-link class="no-underline hover:underline text-gray-800 hover:text-gray-500" to="/oss-license">OSSライセンス表記</router-link>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </footer>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'

export default defineComponent({
  name: 'LandingPage',
  setup () {
    const isHidden = ref(true)
    const navToggle = ref(null as HTMLElement | null)
    const switchNavContentState = (e: Event) => {
      const target = (e && e.target)
      if (!(target instanceof Node)) {
        isHidden.value = true
        return
      }
      const elem = navToggle.value
      if (elem === null) {
        isHidden.value = true
        return
      }
      const result = checkIfTargetWithinElm(target, elem)
      if (result) {
        isHidden.value = !isHidden.value
      } else {
        isHidden.value = true
      }
    }
    return { isHidden, navToggle, switchNavContentState }
  }
})

function checkIfTargetWithinElm (target: Node, elm: Node): boolean {
  let t = target
  while (t.parentNode) {
    if (t === elm) {
      return true
    }
    t = t.parentNode
  }
  return false
}
</script>
