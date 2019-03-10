import init from './mandelbrot.js'

export default function run () {
  const fpsDiv = document.getElementById('fps')
  const canvas = document.getElementById('canvas')
  const width = canvas.width
  const height = canvas.height
  const ctx = canvas.getContext('2d')

  const maxIterations = 150
  const xPos = -0.159998305
  const yPos = 1.04073451103
  let zoom = 0.3

  const averageSize = 30
  const timing = new Array(averageSize)
  let nextPrint = 0
  let frames = 0

  const printStats = now => {
    if (frames > averageSize && nextPrint < now) {
      const averageFrameTime =
        timing.reduce((p, c) => p + c) / averageSize
      const fps = 1000.0 / averageFrameTime
      fpsDiv.innerText = frames + ' frames, ' + fps.toFixed(1) + ' fps'

      nextPrint = now + 2000
    }
  }

  const updateStats = val => {
    frames++
    timing.push(val)
    timing.shift()
  }

  init(width, height, maxIterations)
    .then(res => {
      const { imageData, render } = res

      const draw = () => {
        const tDraw = performance.now()
        const zoom_factor = Math.pow(1.001, tDraw - tLastDraw)
        zoom = zoom * zoom_factor

        updateStats(tDraw - tLastDraw)
        printStats(tDraw)

        tLastDraw = tDraw

        render(xPos, yPos, zoom)

        ctx.putImageData(imageData, 0, 0)
        if (zoom < 3144195305798290) {
          requestAnimationFrame(draw)
        } else {
          const dComplete = performance.now() - tInit
          const averageFrameTime =
            timing.reduce((p, c) => p + c) / averageSize
          const fps = 1000.0 / averageFrameTime

          fpsDiv.innerText = `${frames} frames, ${fps.toFixed(1)} fps, complete in ${dComplete.toFixed(0)} ms`
        }
      }

      const tInit = performance.now()
      let tLastDraw = tInit

      requestAnimationFrame(draw)
    })
}
