import { arch, platform } from 'os'
import { resolve } from 'path'
import registry from 'get-registry'
import { family } from 'detect-libc'
import { Context, Logger, Schema } from 'koishi'
import CanvasService, { Canvas, Image } from '@koishijs/canvas'
import { NereidTask } from 'koishi-plugin-downloads'
import {} from 'koishi-plugin-nix'
import dep from './dep/package.json'

export interface Config {
  nix: boolean
}

export const Config: Schema<Config> = Schema.object({
  nix: Schema.boolean().description('是否使用 koishi-plugin-nix 解决 node 扩展本地依赖问题 (如果你不知道这是什么，请保持关闭)').default(false),
})

export const name = 'canvas'
export const using = ['downloads']

const logger = new Logger(name)

export async function apply(ctx: Context) {
  const task = ctx.downloads.nereid(name, [
    `npm://@koishijs-assets/${dep.name}?registry=${await registry()}`
  ], await bucket())
  if (ctx.config.nix) {
    ctx.using(['nix'], async ctx => {
      if (ctx.nix.avaliable) {
        const pkgs = await ctx.nix.packages(
          'glibc.out', ['glibc', 'libgcc'], ['stdenv.cc.cc', 'lib'], ['fontconfig', 'lib']
        )
        await ctx.nix.patchdir(await task.promise, pkgs.map(pkg => `${pkg}/lib`))
      }
      plugin(ctx, task)
    })
  } else {
    plugin(ctx, task)
  }
}

export async function bucket() {
  return `${dep.name}-v${dep.version}-${platform()}-${arch()}-napi-v6-${await family() || 'unknown'}`
}

async function plugin(ctx: Context, task: NereidTask) {
  const path = await task.promise
  globalThis[`__prebuilt_${dep.name}`] = resolve(process.cwd(), `${path}/v6/index.node`)
  ctx.plugin(NodeCanvasService, require('./dep'))
  logger.info(`${name} started`)
}

export class NodeCanvasService extends CanvasService {
  constructor(ctx: Context, public skia: any) {
    super(ctx)
  }

  async createCanvas(width: number, height: number): Promise<Canvas> {
    return new this.skia.Canvas(width, height)
  }

  async loadImage(source: string | URL | Buffer | ArrayBufferLike): Promise<Image> {
    if (typeof source === 'string' || source instanceof URL) {
      return await this.skia.loadImage(source.toString())
    }
    return this.skia.loadImage(Buffer.from(source))
  }
}
