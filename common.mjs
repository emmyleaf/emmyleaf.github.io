import { spawn } from 'child_process';
import esbuild from 'esbuild';

export const buildScripts = (extraBuildOpts = {}) => esbuild.build({
  entryPoints: ['site_src/script/index.js'],
  outfile: 'build/bundle.js',
  bundle: true,
  format: 'esm',
  minify: true,
  target: ['es6'],
  loader: { '.woff2': 'copy' },
  assetNames: '[name]',
  ...extraBuildOpts
})

export const buildHtml = (cargoRunArgs = []) => {
  return new Promise((resolve, reject) => {
    const process = spawn('cargo', ['run', ...cargoRunArgs], { cwd: 'generator', stdio: 'ignore' })
    process.on('error', err => reject(err))
    process.on('close', () => resolve())
  })
}
