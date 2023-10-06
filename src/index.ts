import { Context, Schema } from 'koishi'
import {} from 'koishi-plugin-downloads'
import {} from 'koishi-plugin-nix'

export interface Config {
  nix: boolean
}

export const Config: Schema<Config> = Schema.object({
  nix: Schema.boolean().description('是否使用 koishi-plugin-nix 解决 node 扩展本地依赖问题 (如果你不知道这是什么，请保持关闭)').default(false),
})

export const name = 'canvas'

export function apply(ctx: Context) {

}