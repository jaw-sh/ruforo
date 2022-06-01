// Note: I want to use SWC, since it's a Rust webpack alternative, but it doesn't have SASS yet.
// https://github.com/swc-project/swc/discussions/4768
// https://swc.rs/docs/configuration/bundling

const { config } = require('@swc/core/spack')


module.exports = config({
    entry: {
        'chat': __dirname + "/resources/js/chat.js",
        'main': __dirname + "/resources/js/attachment.js",
    },
    output: {
        path: __dirname + "/public/assets"
    },
    module: {},
});