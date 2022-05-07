import gluedWasm from './pkg/wasm_node.js';

/**
 * Decodes the png
 * @param bytes the raw bytes from a png image, in Uint8Array
 * @param options options for decoding the image
 * @returns `Uint8Array` of r,g,b,a values for each pixel.
 * This means that the resulting array will have 4 times the size of original image's pixel width * pixel height.
 */
export function decodePng(bytes: Uint8Array, options: ReturnType<typeof createPngDecoderOptions>): Uint8Array {
  return gluedWasm.decode_raw_bytes(bytes, options)
}
/**
 * Just a simple function to avoid writing `new` every single time to create `PngDecoderOptions`
 */
export function createPngDecoderOptions(...params: ConstructorParameters<typeof gluedWasm.PngDecoderOptions>): gluedWasm.PngDecoderOptions {
  return new gluedWasm.PngDecoderOptions(...params);
}
