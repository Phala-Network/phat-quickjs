const path = require('path');

module.exports = {
  entry: './src/index',
  mode: 'production',
  output: {
    filename: 'index.js',
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
      "util": require.resolve("util/"),
      "path": require.resolve("path-browserify"),
      "fs": false
    }
  },
  target: 'node',
  devtool: 'source-map',
};
