import { platform } from 'os'
import { Context, Logger, Schema } from 'koishi'
import registry from 'get-registry'
import { NereidTask } from 'koishi-plugin-downloads'
import {} from 'koishi-plugin-nix'
import { version } from './dep/package.json'
import { resolve } from 'path'

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
    `npm://@koishijs-assets/${name}?registry=${await registry()}`
  ], bucket())
  if (ctx.config.nix) {
    ctx.using(['nix'], async ctx => {
      const pkgs = await ctx.nix.packages('glibc.out', ['libuuid', 'lib'])
      await ctx.nix.patchdir(await task.promise, pkgs.map(pkg => `${pkg}/lib`))
      plugin(ctx, task)
    })
  } else {
    plugin(ctx, task)
  }
}

async function plugin(ctx: Context, task: NereidTask) {
  const path = await task.promise
  globalThis[`__prebuilt_${name}`] = resolve(process.cwd(), `${path}/Release/canvas.node`)
  console.log(require('./dep'))
  logger.info(`${name} started`)
}

export function bucket() {
  let os: string
  switch (platform()) {
    case 'linux':
      os = 'linux-glibc-x64'
      break
    case 'darwin':
      os = 'darwin-unknown-x64'
      break
    case 'win32':
      os = 'win32-unknown-x64'
      break
    default:
      throw new Error('unsupported platform')
  }
  return `${name}-v${version}-node-v${process.versions.modules}-${os}`
}
