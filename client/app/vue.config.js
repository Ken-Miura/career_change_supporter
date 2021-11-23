function removeDataTestAttrs (node) {
  if (node.type === 1 /* NodeTypes.ELEMENT */) {
    node.props = node.props.filter(prop =>
      prop.type === 6 /* NodeTypes.ATTRIBUTE */
        ? prop.name !== 'data-test'
        : true
    )
  }
}

module.exports = {
  configureWebpack: {
    devtool: 'source-map'
  },
  pages: {
    index: {
      entry: 'src/main.ts',
      title: '就職先・転職先を見極めるためのサイト'
    }
  },
  devServer: {
    https: true,
    proxy: {
      '^/api': {
        target: 'http://localhost:3000',
        logLevel: 'debug'
      }
    }
  },
  // chainWebpack内はテストのみで利用するdata-testカスタム属性を取り除く処理
  // vue2用のプラグインはあるが、vue3用には動作しないので直接処理を記載
  // parallel: falseは、npm run buildをパスさせるために必要となる
  // 参考: https://stackoverflow.com/a/67923998/6331122
  parallel: false, // !!IMPORTANT!! - see note below
  chainWebpack: (config) => {
    config.module
      .rule('vue')
      .use('vue-loader')
      .tap((options) => {
        options.compilerOptions = {
          ...(options.compilerOptions || {}),
          nodeTransforms: [removeDataTestAttrs]
        }
        return options
      })
  }
}
