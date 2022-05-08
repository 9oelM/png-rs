import Benchmark from 'benchmark';
import * as PngDecoderWasm from '..'
import path from 'path';
import util from 'util';
import fs from 'fs'
import pngjs from 'pngjs';
const { PNG } = pngjs;

const readFile = util.promisify(fs.readFile);
const readdir = util.promisify(fs.readdir);

const pngFilesDir = path.resolve(__dirname, '..', '..', 'test', 'png', 'official');

function runTestByPngCategories() {

}

(async  () => {
  const pngFileNames = (await readdir(pngFilesDir))
  .filter((filename) => filename.endsWith(`.png`) && !filename.startsWith(`x`))
  .filter((filename) => filename.startsWith('b'))
  
  const pngFileAndPaths = await Promise.all(pngFileNames.map((pngFileName) => new Promise<{ pngFile: Buffer, pngFileName: string  }>(async (resolve, reject) => {
    try {
      const completePath = path.join(pngFilesDir, pngFileName);
      const pngFile = await readFile(completePath)
      
      resolve({ pngFile, pngFileName })
    } catch (e) {
      reject(e)
    }
  })));
  
  const suite = new Benchmark.Suite;
  pngFileAndPaths.forEach(({
    pngFile, pngFileName,
  }) => {
    suite.add(`${pngFileName} with wasm`, () => {
      PngDecoderWasm.decodePng(pngFile, PngDecoderWasm.createPngDecoderOptions(false, true));
    })
    .add(`${pngFileName} with pngjs`, () => {
      PNG.sync.read(pngFile);
    })
  })
  suite
  .on('cycle', function(event: Event) {
    console.log(String(event.target));
  })
  .on('complete', function() {
    console.log(`done`);
  })
  // run async
  .run({ 'async': true });
})()
