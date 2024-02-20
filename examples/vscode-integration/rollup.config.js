import nodeResolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import copy from 'rollup-plugin-copy';
import typescript from "@rollup/plugin-typescript";
import replace from "@rollup/plugin-replace";
import { rawWasm } from '@wesley-clements/rollup-plugin-raw-wasm';

export default {
  input: [ 
    './index.ts'
  ],
  output: {
    dir: './dist',
    format: "cjs",
    sourcemap: true
  },
  external: [
    'vscode'
  ],
  plugins: [
    typescript(),
    rawWasm({
      publicPath: "./dist/",
      copy: true
    }),
    replace({
      values: {
          "import.meta.url": `""`,
      },
      preventAssignment: false,
    }),
    nodeResolve(),
    commonjs({
      transformMixedEsModules: true,
      exclude: [
        "vscode"
      ]
    }),
    copy({
      targets: [
          { src: ['../../pkg/wasmer_js.js', '../../pkg/wasmer_js_bg.wasm'], dest: 'dist' },
      ]
    })
  ]
};