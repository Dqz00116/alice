import type { ToolDef, ToolHandler } from '#src/core/types';

/**
 * Tool 模板
 *
 * 每个 tool 由一个 ToolDef（描述）和一个 ToolHandler（实现）组成。
 *
 * 复制此文件并重命名，填充实现即可。
 */

/** 示例 tool 定义。 */
export const exampleToolDef: ToolDef = {
  name: 'example_tool',
  description: '示例工具',
  input_schema: {
    type: 'object' as const,
    properties: {
      message: { type: 'string' },
    },
    required: ['message'],
  },
};

/** 示例 tool handler。 */
export const exampleToolHandler: ToolHandler = async (args: Record<string, unknown>): Promise<string> => {
  throw new Error('not implemented');
};
