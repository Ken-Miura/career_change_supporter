function removeDataTestAttrs (node) {
  if (node.type === 1 /* NodeTypes.ELEMENT */) {
    node.props = node.props.filter(prop => {
      if (prop.type === 6 /* NodeTypes.ATTRIBUTE */) {
        return prop.name !== 'data-test'
      }
      if (prop.type === 7 /* NodeTypes.DIRECTIVE */) {
        if (prop.arg && prop.arg.content) {
          return prop.arg.content !== 'data-test'
        }
      }
      return true
    })
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
    server: 'https',
    client: {
      webSocketURL: {
        hostname: '0.0.0.0',
        pathname: '/ws',
        protocol: 'wss'
      }
    },
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
