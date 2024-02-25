import nodeResolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import copy from 'rollup-plugin-copy';
import typescript from "@rollup/plugin-typescript";
import replace from "@rollup/plugin-replace";

export default commandLineArgs => {
  return [
    {
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
        })
      ]
    },
    {
      input: [ 
        '../../pkg/wasmer_js.js'
      ],
      output: {
        dir: './dist',
        format: "es",
        sourcemap: true
      },
      external: [
        'vscode'
      ],
      plugins: [
        replace({
          values: {
              "import.meta.url": `""`,
          },
          preventAssignment: false,
        }),
        nodeResolve(),
        copy({
          targets: [
              { src: ['../../dist/wasmer_js_bg.wasm'], dest: 'dist' },
          ]
        })
      ]
    }
  ];
}
