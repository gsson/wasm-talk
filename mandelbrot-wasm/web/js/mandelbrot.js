export default function init (width, height, maxIterations) {
  return WebAssembly.instantiateStreaming(fetch('wasm/mandelbrot.wasm'))
    .then(wasm => {
      const mod = wasm.instance
      const imageBufferOffset = mod.exports.createImageBuffer(width, height)
      const moduleMemory = mod.exports.memory.buffer
      const mandelbrot = mod.exports.mandelbrot

      const pixelArray = new Uint8ClampedArray(
        moduleMemory, imageBufferOffset, width * height * 4)

      const imageData = new ImageData(pixelArray, width, height)
      const render = (posX, posY, zoom) =>
        mandelbrot(imageBufferOffset, width, height, maxIterations, posX, posY, zoom)
      return {
        imageData,
        render
      }
    })
}
