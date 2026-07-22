# CE 项目提示词模板

## 项目定位
- 本项目用于基金监控。
- 当前首个业务模块命名为 `fund-monitor`，路径为 `modules/fund-monitor`。

## 技术栈与模块约定
- 仓库按模块组织，业务模块统一放在 `modules/`。
- 后端与 Web 应用统一使用 Rust。
- Rust Web 框架固定为 `axum`。
- 页面使用原生 `html + css + js`。
- 页面与静态资源默认按“可打包进 Rust 二进制”设计，静态资源优先使用 `rust-embed`；需要服务端模板时优先 `askama`。

## 工作模式
- 本项目默认启用 **Compound Engineering (CE)** 作为主要 AI 工作架构。
- 在没有用户明确要求切换流程的情况下，优先使用 CE 的工作流，避免混入其他并行流程。
- **同一项任务默认只采用一套主工作流。** 若当前任务已明确选择 CE，就不要再混入其他设计/计划/执行流程。
- 若用户明确指定使用其他流程、已有项目规范与 CE 冲突，或当前任务只是一次小型查询/解释，则以用户指令和项目现有规范为准。

## CE 默认工作流
按任务类型优先采用以下顺序：

1. 需求不清、范围未定：`ce:brainstorm` -> `docs/brainstorms/`
2. 需求已清晰、需要计划：`ce:plan` -> `docs/plans/`
3. 进入执行阶段：`ce:work`；需要实验性外部委派时用 `ce:work-beta`
4. 代码改动完成后审查：`ce:review`
5. 问题解决后沉淀：`ce:compound`；历史知识漂移时用 `ce:compound-refresh` -> `docs/solutions/`

## 产物约定
- 需求/产品定义：`docs/brainstorms/`
- 技术计划：`docs/plans/`
- 解决方案/经验沉淀：`docs/solutions/`
- CE 运行期中间产物：`.context/compound-engineering/`

## 执行规则
- 在 CE 工作流中，优先保证：**先澄清，再规划，再执行，再审查，再沉淀**。
- 对于跨文件、跨模块、带有不确定性的任务，不要跳过 `ce:brainstorm` 或 `ce:plan` 直接编码，除非用户明确要求。
- 所有文档中的路径引用都使用**仓库相对路径**，不要使用绝对路径。
- 当任务已经有现成计划文件或 brainstorm 文档时，优先复用和续写，不要重复生成平行文档。
- 若项目中同时存在人工规范、项目 `AGENTS.md`、其他 AI 说明文件，则遵循：
  1. 用户明确指令
  2. 当前项目根目录下的规范文件
  3. CE 工作流约定
  4. 全局默认行为

## Context7 使用准则
- 需要官方库或框架资料时，优先使用 Context7，减少依赖不确定来源的信息。
- 先解析准确的库 ID，再拉取文档；遇到歧义时说明筛选理由。
- 只拉取满足当前问题的最小上下文；Context7 不足时再考虑其他手段。

## Chrome DevTools MCP 使用准则
- 需要排查浏览器端行为、排版或网络问题时，优先使用 `chrome-devtools` MCP。
- 调试前明确目标页面与采集目标；获取结果后整理关键观察并引用输出。
- 若 MCP 不支持所需操作或报错，记录已尝试的命令与错误信息，再改用其他方式。

## Subagent 默认策略
- 任务可拆且写入范围可分离时，默认使用 subagent，不必等用户显式要求。
- 主线程负责拆任务、分配文件 ownership、合并结果、最终验收与 git；子代理负责调查、实现、局部验证。
- 默认并行：多个独立调查点用多个 explorer，多个独立改动块用多个 worker。
- 每个 worker 必须有明确写入范围；不得回滚他人改动，遇到冲突优先适配并汇报。
- 非关键路径任务不要立即等待；只有主线程下一步被阻塞时才 `wait_agent`，完成后及时 `close_agent`。
- 立即阻塞的小任务、强耦合改动、需要连续交互的操作，优先主线程直接处理。
- 所有 subagent 结果最终由主线程统一检查 diff、运行相关测试，并决定是否提交。

## Git 协作简则
- 任何代码、文档、配置调整，只要已经形成稳定结果，都应及时提交并推送。
- 默认一事一提交，只包含当前任务相关改动。
- 小调整默认直接提交并推送；大改动按阶段提交并推送。
- 提交信息默认采用 Conventional Commits 简化格式：`type(scope): summary`
- 常用类型：`feat`、`fix`、`docs`、`refactor`、`chore`
- 推送后只做简要反馈：调整了什么、提交信息、已推送到哪个分支。

<!-- BEGIN COMPOUND CODEX TOOL MAP -->
## Compound Codex Tool Mapping (Claude Compatibility)

This section maps Claude Code plugin tool references to Codex behavior.
Only this block is managed automatically.

Tool mapping:
- Read: use shell reads (cat/sed) or rg
- Write: create files via shell redirection or apply_patch
- Edit/MultiEdit: use apply_patch
- Bash: use shell_command
- Grep: use rg (fallback: grep)
- Glob: use rg --files or find
- LS: use ls via shell_command
- WebFetch/WebSearch: use curl or Context7 for library docs
- AskUserQuestion/Question: present choices as a numbered list in chat and wait for a reply number. For multi-select (multiSelect: true), accept comma-separated numbers. Never skip or auto-configure — always wait for the user's response before proceeding.
- Task/Subagent/Parallel: use Codex subagent/task spawning for splittable work; use multi_tool_use.parallel only for parallel tool calls in the main thread
- TodoWrite/TodoRead: use file-based todos in todos/ with todo-create skill
- Skill: open the referenced SKILL.md and follow it
- ExitPlanMode: ignore
<!-- END COMPOUND CODEX TOOL MAP -->
