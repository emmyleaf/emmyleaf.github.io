import chokidar from 'chokidar';
import { createServer } from 'http';
import serveHandler from 'serve-handler';
import { buildHtml, buildScripts } from './common.mjs';

const runDevBuild = () => buildHtml()
  .then(() => buildScripts({ sourcemap: true, }))
  .then(() => console.log('build complete!'))

runDevBuild().then(() => {
  // start watcher for all source files 
  const watcher = chokidar.watch('site_src');
  watcher.on('ready', () => {
    watcher.on('all', (event) => {
      console.log(`${event} detected, starting build...`)
      runDevBuild()
    })
    console.log('watching...')
  })

  // start server for all build files
  createServer((request, response) => {
    return serveHandler(request, response, { public: 'build' })
  }).listen(8080, () => {
    console.log('serving at http://localhost:8080')
  })
})
