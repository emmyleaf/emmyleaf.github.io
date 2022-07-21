import { buildHtml, buildScripts } from './common.mjs';

buildHtml(['--release'])
  .then(() => buildScripts())
  .then(() => console.log('build complete!'), (err) => console.error(`build failed! ${err}`))
