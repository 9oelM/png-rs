import fs from 'fs';
import pkg from 'pngjs';
import to from 'await-to-js';
import * as PngWasm from '.';
const { PNG } = pkg;

test.concurrent.each(global.allPngInfo)(`should decode $path from correctly`, async ({ file, path }) => {
  const [err, jsOutput] = await to<Uint8Array, Error>(new Promise((resolve, reject) => {
    try {
      fs.createReadStream(path)
        .pipe(new PNG({
        }))
        .on("parsed", function (data) {
          resolve(new Uint8Array(data));
        })
        .on("error", (err) => {
          throw err;
        })
    } catch (e) {
      reject(e)
    }
  }));

  if (jsOutput && !err) {
    const wasmOutput = PngWasm.decodePng(file, PngWasm.createPngDecoderOptions(false, true));
    if (wasmOutput.length !== jsOutput.length) {
      console.error(`jsOutput and wasmOutput have different lengths for ${path}: ${wasmOutput.length}, ${jsOutput.length}`)
    } else {
      expect(wasmOutput.length).toStrictEqual(jsOutput.length);
      expect(wasmOutput).toStrictEqual(jsOutput);
    }
  } else if (err) {
    console.error(`Testing ${path} caused an error in pngjs: ${JSON.stringify(err.message)}`)
  }
  console.log(`Testing ${path} done`);
  return Promise.resolve()
});