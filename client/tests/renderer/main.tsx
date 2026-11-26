import { renderToStaticMarkup } from 'react-dom/server'
import { Illustration } from '../../src/TopDownIllustration.tsx'
import { BACKDROP, CLIP, VIEWPORT, SEED, texelsPerMetre, scene, pose, hit, packAtlases, distribution,
  type Asset, type BitmapSize, type Density, type DrawObject, type FrameObject, type Renderer, type Scenario } from './model.ts'

const SVG = 'http://www.w3.org/2000/svg'
const root = document.querySelector<HTMLElement>('#scene')!
const status = document.querySelector<HTMLElement>('#status')!
type MeasuredAsset = BitmapSize & { canvas: HTMLCanvasElement; metres: readonly [number, number]; url: string; pngBytes: number }
interface Adapter {
  backend: string
  render(frame: readonly FrameObject[]): number | null
  dispose(): void
  loseContext?(): boolean
  memory: number
  pages: BitmapSize[]
  nodeCount: number
  compressedTextures: number
}

const artwork: Record<Asset, { metres: readonly [number, number]; fragment: string }> = {
  hull: { metres: [8, 14.6], fragment: renderToStaticMarkup(<Illustration appearance="hull"/>) },
  deck: { metres: [6, 13.6], fragment: renderToStaticMarkup(<Illustration appearance="floor"/>) },
  chair: { metres: [1.1, 1.1], fragment: renderToStaticMarkup(<Illustration appearance="chair"/>) },
  core: { metres: [1.2, 1.8], fragment: renderToStaticMarkup(<Illustration appearance="core"/>) },
  bench: { metres: [1.6, 1.7], fragment: renderToStaticMarkup(<Illustration appearance="fabricator"/>) },
  storage: { metres: [1.5, 1.8], fragment: renderToStaticMarkup(<Illustration appearance="storage"/>) },
  bed: { metres: [1.2, 2.8], fragment: renderToStaticMarkup(<Illustration appearance="bed"/>) },
  terrain: {
    metres: [8, 8],
    fragment: '<rect x="-4" y="-4" width="8" height="8" fill="#b5a58b"/><path fill="none" stroke="#9d8d73" stroke-opacity=".6" stroke-width=".04" d="M-3.8 2Q-2 -1 2 .3L3.8 -1M-3.8 2.3Q-2 -.7 2 .6L3.8 -.7M-3.6 2.6Q-2 -.4 2 .9L3.8 -.4"/><path fill="none" stroke="#d7cab0" stroke-width=".06" d="M-2 -2.5l.45 .1m2 4.4l.3 -.1m1.1 .6l.45 -.2m-2 -5.4l.6 .1"/>',
  },
  stone: { metres: [1.4, 1.2], fragment: renderToStaticMarkup(<Illustration appearance="stones"/>) },
  mineral: { metres: [2.8, 2.2], fragment: renderToStaticMarkup(<Illustration appearance="mineral"/>) },
  flora: { metres: [2.8, 2.2], fragment: renderToStaticMarkup(<Illustration appearance="flora"/>) },
  helmet: { metres: [.7, .7], fragment: '<ellipse rx=".30" ry=".27" fill="#e2d8b9" stroke="#344950" stroke-width=".05"/><path d="M-.2 -.12Q0 -.26 .2 -.12" stroke="#86d7d7" fill="none" stroke-width=".08"/>' },
  torso: { metres: [.8, .8], fragment: '<rect x="-.31" y="-.29" width=".62" height=".58" rx=".17" fill="#425962" stroke="#344950" stroke-width=".05"/><rect x="-.13" y="-.07" width=".26" height=".20" rx=".05" fill="#a8cac6"/>' },
  arm: { metres: [.6, .6], fragment: '<rect x="-.10" y="-.23" width=".20" height=".46" rx=".08" fill="#596b71" stroke="#293c43" stroke-width=".04"/>' },
  boot: { metres: [.5, .5], fragment: '<rect x="-.10" y="-.17" width=".20" height=".34" rx=".07" fill="#344950"/>' },
  light: { metres: [4, 4], fragment: '<defs><radialGradient id="warm"><stop stop-color="#ffe3a7" stop-opacity=".66"/><stop offset="1" stop-color="#ffe3a7" stop-opacity="0"/></radialGradient></defs><circle r="2" fill="url(#warm)"/>' },
  particle: { metres: [.32, .32], fragment: '<circle r=".09" fill="#fff1cb"/>' },
}

const png = (canvas: HTMLCanvasElement) => new Promise<Blob>((resolve, reject) => {
  canvas.toBlob(blob => blob ? resolve(blob) : reject(new Error('PNG encoding failed')), 'image/png')
})
function bitmapCanvas(width: number, height: number) {
  const canvas = document.createElement('canvas')
  canvas.width = width
  canvas.height = height
  const context = canvas.getContext('2d')
  if (!context) throw Error('Canvas 2D unavailable')
  return { canvas, context }
}
async function rasterize(asset: Asset, density: Density): Promise<MeasuredAsset> {
  const { metres, fragment } = artwork[asset], [w, h] = metres
  const resolution = texelsPerMetre(asset, density)
  const width = Math.ceil(w * resolution), height = Math.ceil(h * resolution)
  const svg = '<svg xmlns="' + SVG + '" width="' + width + '" height="' + height + '" viewBox="' + [-w / 2, -h / 2, w, h].join(' ') + '">' + fragment + '</svg>'
  const source = URL.createObjectURL(new Blob([svg], { type: 'image/svg+xml' }))
  try {
    const image = new Image()
    image.src = source
    await image.decode()
    const { canvas, context } = bitmapCanvas(width, height)
    context.drawImage(image, 0, 0)
    const blob = await png(canvas)
    return { canvas, width, height, metres, url: URL.createObjectURL(blob), pngBytes: blob.size }
  } finally {
    URL.revokeObjectURL(source)
  }
}
const rgba = (sizes: readonly BitmapSize[]) => sizes.reduce((sum, { width, height }) => sum + width * height * 4, 0)
const update = (node: Element, changes: Record<string, string | number>) => {
  for (const [key, value] of Object.entries(changes)) node.setAttribute(key, String(value))
}

function svgAdapter(objects: readonly DrawObject[], images: ReadonlyMap<Asset, MeasuredAsset>): Adapter {
  const viewport = document.createElementNS(SVG, 'svg')
  update(viewport, { width: VIEWPORT.width, height: VIEWPORT.height, viewBox: '0 0 ' + VIEWPORT.width + ' ' + VIEWPORT.height })
  const clip = document.createElementNS(SVG, 'clipPath')
  clip.id = 'scene-clip'
  const bounds = document.createElementNS(SVG, 'rect')
  update(bounds, CLIP)
  clip.append(bounds)
  const stage = document.createElementNS(SVG, 'g')
  update(stage, { 'clip-path': 'url(#scene-clip)' })
  viewport.append(clip, stage)
  const nodes = objects.map(obj => {
    const img = document.createElementNS(SVG, 'image')
    const asset = images.get(obj.asset)!
    const [w, h] = asset.metres.map(m => m * obj.zoom)
    update(img, { href: asset.url, x: -w! / 2, y: -h! / 2, width: w!, height: h!, preserveAspectRatio: 'none' })
    stage.append(img)
    return img
  })
  root.replaceChildren(viewport)
  let initialized = false
  return {
    backend: 'SVG image elements; Chromium compositor',
    render(frame) {
      frame.forEach((obj, index) => {
        if (!initialized || obj.motion) {
          update(nodes[index]!, { transform: 'translate(' + obj.x + ' ' + obj.y + ') rotate(' + obj.rotation + ')', opacity: obj.alpha })
        }
      })
      initialized = true
      return null
    },
    dispose: () => viewport.remove(),
    memory: rgba([...images.values()]), pages: [],
    nodeCount: nodes.length, compressedTextures: [...images.values()].reduce((sum, asset) => sum + asset.pngBytes, 0),
  }
}

async function pixiAdapter(objects: readonly DrawObject[], images: ReadonlyMap<Asset, MeasuredAsset>, listeners: AbortSignal): Promise<Adapter> {
  const { Application, Container, Graphics, Rectangle, Sprite, Texture } = await import('pixi.js')
  const sizes = [...images].map(([key, { width, height }]) => ({ key, width, height }))
  const { entries, pages } = packAtlases(sizes)
  const atlas = pages.map(({ width, height }) => bitmapCanvas(width, height))
  for (const [key, { page, x, y }] of entries) atlas[page]!.context.drawImage(images.get(key)!.canvas, x, y)
  const compressedTextures = (await Promise.all(atlas.map(({ canvas }) => png(canvas))))
    .reduce((sum, blob) => sum + blob.size, 0)
  const textures = atlas.map(({ canvas }) => Texture.from(canvas))
  const frames = new Map([...entries].map(([key, { page, x, y, width, height }]) =>
    [key, new Texture({ source: textures[page]!.source, frame: new Rectangle(x, y, width, height) })]))
  const canvas = document.createElement('canvas')
  canvas.addEventListener('webglcontextlost', () => {
    root.dataset.backendFailed = 'true'
    status.textContent = 'WebGL context lost. Reload the benchmark.'
  }, { signal: listeners })
  const gl = canvas.getContext('webgl2', { alpha: false, antialias: false, preserveDrawingBuffer: true })
  if (!gl) {
    textures.forEach(texture => texture.destroy(true))
    throw Error('WebGL2 unavailable')
  }
  const info = gl.getExtension('WEBGL_debug_renderer_info')
  const backend = String(gl.getParameter(info?.UNMASKED_RENDERER_WEBGL ?? gl.RENDERER))
  let draws = 0
  for (const key of ['drawArrays', 'drawElements', 'drawArraysInstanced', 'drawElementsInstanced'] as const) {
    const draw = gl[key]
    Reflect.set(gl, key, (...args: unknown[]) => { draws++; return Reflect.apply(draw, gl, args) })
  }
  const app = new Application()
  await app.init({
    canvas, context: gl, width: VIEWPORT.width, height: VIEWPORT.height, resolution: devicePixelRatio,
    background: BACKDROP, backgroundAlpha: 1, preference: 'webgl', preferWebGLVersion: 2,
    autoStart: false, antialias: false, useBackBuffer: true,
  })
  const stage = new Container(), mask = new Graphics().rect(CLIP.x, CLIP.y, CLIP.width, CLIP.height).fill('#fff')
  stage.mask = mask
  app.stage.addChild(stage, mask)
  const sprites = objects.map(obj => {
    const sprite = new Sprite(frames.get(obj.asset)!)
    sprite.anchor.set(.5)
    const [w, h] = images.get(obj.asset)!.metres
    sprite.width = w * obj.zoom
    sprite.height = h * obj.zoom
    stage.addChild(sprite)
    return sprite
  })
  root.replaceChildren(canvas)
  let initialized = false
  return {
    backend,
    render(frame) {
      if (gl.isContextLost()) throw Error('WebGL context lost')
      frame.forEach((obj, index) => {
        if (!initialized || obj.motion) {
          const sprite = sprites[index]!
          sprite.position.set(obj.x, obj.y)
          sprite.angle = obj.rotation
          sprite.alpha = obj.alpha
        }
      })
      initialized = true
      draws = 0
      app.render()
      return draws
    },
    dispose() {
      app.destroy({ removeView: true }, { children: true })
      for (const frame of frames.values()) frame.destroy(false)
      for (const texture of textures) texture.destroy(true)
    },
    loseContext() {
      const extension = gl.getExtension('WEBGL_lose_context')
      extension?.loseContext()
      return !!extension
    },
    memory: rgba(pages), pages, nodeCount: sprites.length, compressedTextures,
  }
}

export interface BenchmarkConfig { renderer: Renderer; scenario: Scenario; density: Density }
async function mount(config: BenchmarkConfig) {
  if (!['svg', 'pixi'].includes(config.renderer) || !['scout', 'surface'].includes(config.scenario)
    || !['current', '128'].includes(config.density)) throw Error('Unsupported renderer benchmark configuration')
  const started = performance.now()
  const objects = scene(config.scenario)
  const assets = [...new Set(objects.map(object => object.asset))]
  const bitmaps = new Map(await Promise.all(assets.map(async asset => [asset, await rasterize(asset, config.density)] as const)))
  const listeners = new AbortController()
  let adapter: Adapter
  try {
    adapter = config.renderer === 'pixi' ? await pixiAdapter(objects, bitmaps, listeners.signal) : svgAdapter(objects, bitmaps)
  } catch (error) {
    listeners.abort()
    for (const bitmap of bitmaps.values()) URL.revokeObjectURL(bitmap.url)
    throw error
  }
  root.dataset.pick = 'none'
  delete root.dataset.backendFailed
  status.textContent = ''
  root.addEventListener('pointerdown', event => {
    const box = root.getBoundingClientRect()
    root.dataset.pick = hit(objects, (event.clientX - box.x) * VIEWPORT.width / box.width,
      (event.clientY - box.y) * VIEWPORT.height / box.height) ?? 'none'
  }, { signal: listeners.signal })
  let disposed = false
  const renderAt = (frame: number) => {
    if (disposed) throw Error('Disposed renderer')
    return adapter.render(objects.map(object => pose(object, frame)))
  }
  renderAt(0)
  const prepareMs = performance.now() - started
  const estimate = {
    ...config, backend: adapter.backend, seed: SEED, viewport: VIEWPORT, dpr: devicePixelRatio,
    prepareMs, objectCount: objects.length, nodeCount: adapter.nodeCount, sourceImages: imagesSummary(bitmaps),
    compressedTextures: adapter.compressedTextures, rgbaBytes: adapter.memory,
    rgbaWithMipsUpperBound: Math.ceil(adapter.memory * 4 / 3),
    atlasPages: adapter.pages, rasterStagingBytes: rgba([...bitmaps.values()]),
    atlasStagingBytes: config.renderer === 'pixi' ? adapter.memory : 0,
    estimatedFrameBufferBytes: config.renderer === 'pixi'
      ? 3 * VIEWPORT.width * VIEWPORT.height * devicePixelRatio ** 2 * 4 : null,
  }
  return {
    renderAt, estimate,
    async measure() {
      const intervals: number[] = [], submission: number[] = [], drawCalls: number[] = []
      let previous = 0
      // Every trial samples the same logical frame sequence on real rAF. Record
      // CPU submission separately because GPU queue completion is asynchronous.
      for (let frame = 0; frame <= 110; frame++) {
        const now = await new Promise<number>(resolve => requestAnimationFrame(resolve))
        if (document.hidden) throw Error('Benchmark sample requires a visible page')
        const start = performance.now()
        const draws = renderAt(frame)
        if (frame > 20) {
          intervals.push(now - previous)
          submission.push(performance.now() - start)
          if (draws !== null) drawCalls.push(draws)
        }
        previous = now
      }
      return {
        estimate, intervals, submission, drawCalls,
        frameMs: distribution(intervals), submissionMs: distribution(submission),
        drawsPerFrame: drawCalls.length ? distribution(drawCalls) : null,
        sustainedFps: 1000 * intervals.length / intervals.reduce((sum, ms) => sum + ms, 0),
      }
    },
    loseContext: () => adapter.loseContext?.() ?? false,
    dispose() {
      if (disposed) return
      disposed = true
      listeners.abort()
      adapter.dispose()
      for (const bitmap of bitmaps.values()) URL.revokeObjectURL(bitmap.url)
    },
  }
}
function imagesSummary(images: ReadonlyMap<Asset, MeasuredAsset>) {
  return [...images].map(([asset, { width, height, pngBytes }]) => ({ asset, width, height, pngBytes }))
}

let current: Awaited<ReturnType<typeof mount>> | undefined
export const rendererBenchmark = {
  async mount(config: BenchmarkConfig) {
    current?.dispose()
    current = undefined
    try { current = await mount(config); return current.estimate }
    catch (error) { status.textContent = error instanceof Error ? error.message : 'Renderer initialization failed'; throw error }
  },
  renderAt: (frame: number) => current!.renderAt(frame),
  measure: () => current!.measure(),
  loseContext: () => current!.loseContext(),
  dispose() { current?.dispose(); current = undefined },
}
Object.assign(window, { rendererBenchmark })
