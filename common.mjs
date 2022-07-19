import { spawn } from 'child_process';
import esbuild from 'esbuild';

const baseBuildOptions = {
  entryPoints: ['site_src/script/index.js'],
  outfile: 'build/bundle.js',
  bundle: true,
  format: 'esm',
  minify: true,
  target: ['es6'],
  loader: { '.woff2': 'copy' },
  assetNames: '[name]',
}

export const buildScripts = (extraBuildOpts = {}) => esbuild.build({ ...baseBuildOptions, ...extraBuildOpts })

export const buildHtml = (cargoRunArgs = []) => {
  return new Promise((resolve, reject) => {
    const process = spawn('cargo', ['run', ...cargoRunArgs], { cwd: 'leafcodes_ssg', stdio: 'ignore' })
    process.on('error', err => reject(err))
    process.on('close', () => resolve())
  })
}
