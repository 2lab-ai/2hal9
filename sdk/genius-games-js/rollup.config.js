import typescript from '@rollup/plugin-typescript';
import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import json from '@rollup/plugin-json';
import { terser } from 'rollup-plugin-terser';

const production = !process.env.ROLLUP_WATCH;

export default [
  // ES Module build
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.esm.js',
      format: 'es',
      sourcemap: true
    },
    external: ['eventemitter3', 'isomorphic-ws', 'ws'],
    plugins: [
      resolve(),
      commonjs(),
      json(),
      typescript({
        tsconfig: './tsconfig.json',
        declaration: true,
        declarationDir: 'dist',
        rootDir: 'src'
      }),
      production && terser()
    ]
  },
  
  // CommonJS build
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.js',
      format: 'cjs',
      sourcemap: true
    },
    external: ['eventemitter3', 'isomorphic-ws', 'ws'],
    plugins: [
      resolve(),
      commonjs(),
      json(),
      typescript({
        tsconfig: './tsconfig.json'
      }),
      production && terser()
    ]
  },
  
  // UMD build for browsers
  {
    input: 'src/index.ts',
    output: {
      file: 'dist/index.umd.js',
      format: 'umd',
      name: 'GeniusGames',
      sourcemap: true,
      globals: {
        'eventemitter3': 'EventEmitter3',
        'isomorphic-ws': 'WebSocket'
      }
    },
    external: ['ws'], // Exclude Node.js WebSocket
    plugins: [
      resolve({
        browser: true,
        preferBuiltins: false
      }),
      commonjs(),
      json(),
      typescript({
        tsconfig: './tsconfig.json'
      }),
      production && terser()
    ]
  }
];