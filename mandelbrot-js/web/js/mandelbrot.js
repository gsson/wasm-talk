export default function init (width, height, maxIterations) {
  const byteSize = width * height * 4
  const image = new Uint8ClampedArray(byteSize)
  const imageData = new ImageData(image, width, height)
  const render = (posX, posY, zoom) =>
    mandelbrot(image, width, height,
      maxIterations, posX, posY, zoom)

  return Promise.resolve({
    imageData,
    render
  })
}

function mandelbrot (image, width, height,
                     maxIterations, posX, posY, zoom) {
  const aspect = height / width
  const halfZoom = zoom / 2.0
  const zoomedHeight = halfZoom * height
  const zoomedWidth = halfZoom * width * aspect
  const halfWidth = width / 2
  const halfHeight = height / 2
  let index = 0

  const maxIterationsSquared = maxIterations * maxIterations

  for (let y = -halfHeight; y < halfHeight; y++) {
    const y0 = y / zoomedHeight + posY

    for (let x = -halfWidth; x < halfWidth; x++) {
      const x0 = x / zoomedWidth + posX
      const iterations = iterate(x0, y0, maxIterations)

      const c = iterations * iterations * 255 / maxIterationsSquared
      image[index] = c
      image[index + 1] = c
      image[index + 2] = c
      image[index + 3] = 0xff

      index += 4
    }
  }
}

function iterate (x0, y0, maxIterations) {
  let real = 0
  let imaginary = 0

  for (let i = 0; i < maxIterations; i++) {
    const realSquared = real * real
    const imaginarySquared = imaginary * imaginary

    if (realSquared + imaginarySquared >= 4.0) {
      return i
    }

    const realTemp = realSquared - imaginarySquared + x0
    imaginary = 2.0 * real * imaginary + y0
    real = realTemp
  }
  return maxIterations
}
