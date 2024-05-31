const path = require('path');
const webpack = require('webpack');

module.exports = {
  entry: './src/index',
  mode: 'production',
  output: {
    filename: 'index.js',
    path: path.resolve(__dirname, 'dist'),
    publicPath: '',
    globalObject: 'globalThis',
    scriptType: false,
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
      fs: false,
      // canvas: false,
      child_process: false,
      net: false,
      tls: false,

      assert: require.resolve("assert/"),
      http: require.resolve("stream-http"),
      https: require.resolve("https-browserify"),
      vm: require.resolve("vm-browserify"),
      util: require.resolve("util/"),
      buffer: require.resolve("buffer/"),
      url: require.resolve("url/"),
      os: require.resolve("os-browserify/browser"),
      zlib: require.resolve("browserify-zlib"),
      stream: require.resolve("stream-browserify"),
      // crypto: require.resolve("crypto-browserify"),
      path: require.resolve("path-browserify"),
      crypto: false,
    },
  },
  plugins: [
    new webpack.ProvidePlugin({
      Buffer:  ['buffer', 'Buffer'],
    }),
  ],
  target: 'web',
  devtool: 'source-map',
};
