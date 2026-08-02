# CE 项目提示词模板

## 项目定位
- 本项目用于基金监控。
- 当前首个业务模块命名为 `fund-monitor`，路径为 `modules/fund-monitor`。
- 当前已新增桌面端模块 `app`，路径为 `modules/app`。

## 技术栈与模块约定
- 仓库按模块组织，业务模块统一放在 `modules/`。
- `modules/fund-monitor` 作为 Rust 核心后台模块，负责数据抓取、规则计算、任务调度、通知与存储，不再承载 Web UI。
- 所有前端开发统一在 `modules/app` 进行。
- `modules/app` 使用 Flutter 开发，当前先支持 `Windows / macOS / Linux`，未来再补 `Android / iOS`。

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

## Git 协作简则
- 任何代码、文档、配置调整，只要已经形成稳定结果，都应及时提交并推送。
- 默认一事一提交，只包含当前任务相关改动。
- 小调整默认直接提交并推送；大改动按阶段提交并推送。
- 提交信息默认采用 Conventional Commits 简化格式：`type(scope): summary`
- 常用类型：`feat`、`fix`、`docs`、`refactor`、`chore`
- 推送后只做简要反馈：调整了什么、提交信息、已推送到哪个分支。
