const path = require('path');
const MiniCssExtractPlugin = require("mini-css-extract-plugin");

module.exports = {
    mode: 'development',
    experiments: {
        asyncWebAssembly: true,
    },
    entry: {
        main: './resources/js/attachments.js',
        style: './resources/css/main.scss',
    },
    output: {
        path: path.resolve(__dirname, 'public/assets'),
        publicPath: '/assets/',
        filename: '[name].js',
        clean: true,
    },
    module: {
        rules: [
            {
                test: /\.js$/,
                enforce: "pre",
                use: ["source-map-loader"],
            },
            {
                test: /\.s[ac]ss$/i,
                use: [
                  "style-loader",
                  MiniCssExtractPlugin.loader,
                  "css-loader",
                  "sass-loader",
                ],
            },
        ],
    },
    plugins: [
        new MiniCssExtractPlugin({
            filename: "[name].css",
            chunkFilename: "[id].css",
        }),
    ],
};
