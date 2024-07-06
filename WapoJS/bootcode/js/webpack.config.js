const path = require('path');

module.exports = {
  entry: {
    'browser': './src/browser',
    'nodejs': './src/nodejs',
    'wapo': './src/wapo',
  },
  mode: 'production',
  output: {
    filename: '[name].js',
    path: path.resolve(__dirname, 'dist'),
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
    fallback: {
      "assert/strict": jspm_module('assert/strict.js'),
      assert: jspm_module('assert.js'),
      async_hooks: jspm_module('async_hooks.js'),
      buffer: jspm_module('buffer.js'),
      child_process: jspm_module('child_process.js'),
      cluster: jspm_module('cluster.js'),
      console: jspm_module('console.js'),
      constants: jspm_module('constants.js'),
      crypto: require.resolve('crypto-browserify'),
      diagnostics_channel: jspm_module('diagnostics_channel.js'),
      dns: jspm_module('dns.js'),
      domain: jspm_module('domain.js'),
      events: jspm_module('events.js'),
      'fs/promises': jspm_module('fs/promises.js'),
      fs: jspm_module('fs.js'),
      http: require.resolve('stream-http'),
      https: jspm_module('https.js'),
      module: jspm_module('module.js'),
      net: jspm_module('net.js'),
      os: jspm_module('os.js'),
      path: jspm_module('path.js'),
      perf_hooks: jspm_module('perf_hooks.js'),
      process: jspm_module('process.js'),
      punycode: jspm_module('punycode.js'),
      querystring: jspm_module('querystring.js'),
      readline: jspm_module('readline.js'),
      repl: jspm_module('repl.js'),
      stream: require.resolve('readable-stream'),
      string_decoder: jspm_module('string_decoder.js'),
      sys: jspm_module('sys.js'),
      'timers/promises': jspm_module('timers/promises.js'),
      timers: jspm_module('timers.js'),
      tls: jspm_module('tls.js'),
      tty: jspm_module('tty.js'),
      url: jspm_module('url.js'),
      util: jspm_module('util.js'),
      v8: jspm_module('v8.js'),
      vm: jspm_module('vm.js'),
      wasi: jspm_module('wasi.js'),
      worker_threads: jspm_module('worker_threads.js'),
      zlib: jspm_module('zlib.js'),
    }
  },
  target: 'web',
  devtool: 'source-map',
};

function jspm_module(module) {
  const modulePath = path.resolve(__dirname, 'node_modules', '@jspm/core/nodelibs/browser', module);
  console.log(`jspm_module: ${module} => ${modulePath}`);
  return modulePath;
}