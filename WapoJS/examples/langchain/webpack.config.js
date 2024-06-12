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
    fallback: {},
  },
  plugins: [
    new webpack.ProvidePlugin({
      Buffer:  ['buffer', 'Buffer'],
    }),
  ],
  target: 'node',
  devtool: 'source-map',
};
