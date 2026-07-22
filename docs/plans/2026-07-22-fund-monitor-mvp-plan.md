# 基金监控模块 MVP 计划

日期：2026-07-22

## 1. 目标

在 `modules/fund-monitor` 中落地一个基金监控模块，提供基金池管理、数据采集、规则监控、告警通知和基础看板能力。

首期目标：
- 能维护监控基金列表
- 能定时抓取基金关键数据
- 能按规则判断是否触发告警
- 能查看最近监控状态与告警结果
- 页面和静态资源可直接打包进 Rust 二进制

## 2. 首期范围

### 2.1 必做
- 基金池管理
  - 添加基金
  - 删除基金
  - 编辑备注
  - 支持分组或标签
- 基金数据采集
  - 拉取基金基础信息
  - 拉取净值 / 估值 / 涨跌幅
  - 记录最近抓取时间和抓取状态
- 监控规则管理
  - 涨跌幅阈值规则
  - 净值区间规则
  - 估值偏离规则
  - 规则启用 / 停用
- 定时监控任务
  - 定时轮询
  - 执行规则判断
  - 生成告警事件
  - 短时间重复告警抑制
- 告警能力
  - 站内告警列表
  - 先接入一种外部通知渠道
  - 告警状态管理：新告警 / 已处理 / 已忽略
- 监控面板
  - 基金列表页
  - 基金详情页
  - 告警列表页
  - 总览页
  - 系统配置页
- 历史记录
  - 基金数据历史
  - 告警历史
  - 任务执行历史

### 2.2 暂不纳入首期
- 多用户和权限体系
- 复杂统计分析和收益图表
- 多通知渠道同时投递
- 导入导出
- 自定义表达式规则
- 组合收益分析

## 3. 技术方案

### 3.1 固定技术栈
- 后端：Rust
- Web 框架：`axum`
- 页面：原生 `html + css + js`
- 服务端模板：优先 `askama`
- 静态资源嵌入：`rust-embed`
- 异步运行时：`tokio`

### 3.2 存储建议
- 首期建议使用 `SQLite`
- 原因：
  - 更适合单机部署和 MVP 快速落地
  - 与单二进制目标更匹配
  - 便于后续迁移到更重的数据库

### 3.3 模块目录建议

以 `modules/fund-monitor` 为中心组织：

```text
modules/fund-monitor/
├── src/
│   ├── main.rs
│   ├── app/
│   ├── web/
│   ├── domain/
│   ├── storage/
│   ├── providers/
│   ├── jobs/
│   └── notifications/
├── web/
│   ├── index.html
│   ├── app.css
│   └── app.js
└── migrations/
```

建议职责：
- `app/`：应用启动、配置、依赖装配
- `web/`：路由、handler、页面返回
- `domain/`：基金、规则、告警等领域模型
- `storage/`：数据库访问、仓储实现
- `providers/`：基金数据源抽象与实现
- `jobs/`：轮询任务、规则执行、重试逻辑
- `notifications/`：外部通知渠道

## 4. 核心数据对象

首期建议先定义以下核心对象：

- `fund`
  - 基金代码
  - 名称
  - 备注
  - 分组 / 标签
  - 启用状态
- `fund_quote`
  - 基金 ID
  - 净值
  - 估值
  - 涨跌幅
  - 抓取时间
  - 数据源
- `monitor_rule`
  - 规则类型
  - 阈值配置
  - 关联基金或分组
  - 启用状态
  - 冷却时间
- `alert_event`
  - 关联规则
  - 关联基金
  - 触发原因
  - 告警状态
  - 触发时间
- `job_run`
  - 任务类型
  - 开始时间
  - 结束时间
  - 结果状态
  - 错误信息
- `app_setting`
  - 轮询频率
  - 数据源配置
  - 通知渠道配置

## 5. 页面规划

- `/dashboard`
  - 今日概览
  - 抓取状态
  - 告警摘要
- `/funds`
  - 基金列表
  - 分组筛选
  - 添加 / 编辑 / 删除
- `/funds/:id`
  - 基金详情
  - 最近数据
  - 历史数据
  - 关联规则
- `/rules`
  - 规则列表
  - 创建 / 启停 / 删除
- `/alerts`
  - 告警列表
  - 状态处理
- `/settings`
  - 轮询频率
  - 数据源
  - 通知渠道

## 6. 数据流

### 6.1 采集流
1. 定时任务触发
2. 拉取基金数据
3. 写入 `fund_quote`
4. 记录 `job_run`

### 6.2 监控流
1. 读取最新基金数据
2. 读取启用规则
3. 执行规则判断
4. 生成 `alert_event`
5. 触发站内和外部通知

### 6.3 页面流
1. 页面请求进入 `axum`
2. 由 handler 读取数据库
3. 用模板或 HTML 片段生成页面
4. 静态资源通过 `rust-embed` 提供

## 7. 分阶段实施

### Phase 1：基础骨架
- 完成模块骨架
- 完成页面壳和导航
- 完成配置加载
- 完成数据库接入
- 完成基础表结构

验收：
- 应用可启动
- 页面可访问
- 数据库可初始化

### Phase 2：基金池与采集
- 完成基金 CRUD
- 接入首个基金数据源
- 完成定时抓取
- 写入历史数据

验收：
- 可新增基金并抓取到数据
- 能看到最近一次抓取结果

### Phase 3：规则与告警
- 完成规则管理
- 完成规则执行逻辑
- 完成告警事件生成
- 接入首个外部通知渠道

验收：
- 规则触发后可生成告警
- 至少一种外部通知可用

### Phase 4：面板与收口
- 完成总览页
- 完成告警列表和处理状态
- 完成历史记录展示
- 完成基础错误处理和日志

验收：
- 可通过页面查看基金状态、历史和告警

## 8. 首期实现建议

建议优先顺序：
1. 基础表结构和配置
2. 基金池管理
3. 数据采集
4. 规则执行
5. 告警通知
6. 看板页面

建议首个通知渠道：
- `Telegram Bot`

原因：
- 接入成本低
- 适合个人基金监控场景
- 便于快速验证告警链路

## 9. 风险与待确认项

- 基金数据源选择与稳定性
- 估值数据是否实时、是否存在延迟
- 通知渠道最终采用 Telegram、邮件还是 webhook
- 定时任务在单实例部署下是否足够，后续是否需要独立 worker
- 历史数据保留策略和清理策略

## 10. 建议下一步

建议按以下顺序继续：
1. 先补一份需求 / 方案确认文档，冻结首期范围
2. 细化数据库表结构和模块目录
3. 拆解为可执行任务清单
4. 进入 `fund-monitor` 模块实现

## 11. Implementation Units

### Unit 1：应用骨架、配置与数据库初始化

**Goal:**  
为 `fund-monitor` 建立稳定的应用骨架，接入配置加载、SQLite 初始化和基础迁移入口。

**Files:**  
- `modules/fund-monitor/Cargo.toml`
- `modules/fund-monitor/src/main.rs`
- `modules/fund-monitor/src/app/mod.rs`
- `modules/fund-monitor/src/app/config.rs`
- `modules/fund-monitor/src/app/state.rs`
- `modules/fund-monitor/src/storage/mod.rs`
- `modules/fund-monitor/src/storage/db.rs`
- `modules/fund-monitor/migrations/`
- `modules/fund-monitor/.env.example`

**Approach:**  
- 保留 `axum` 作为入口，拆出 `app` 和 `storage` 层。  
- 首期使用 SQLite，数据库文件路径通过配置注入。  
- 启动时完成数据库连接与迁移执行。  
- 将配置、数据库连接池和后续服务依赖统一挂到应用状态中。

**Test scenarios:**  
- 启动应用时，在空目录下创建数据库文件并完成迁移初始化。  
- 配置缺失时，应用启动失败并返回明确错误。  
- 数据库路径不可写时，应用启动失败并记录错误原因。  
- Test expectation: 页面样式和纯静态资源本单元不新增行为测试。

**Verification:**  
- 应用可在本地稳定启动。  
- 数据库可自动初始化到首个可用状态。  
- 配置读取和应用状态装配路径清晰可扩展。

### Unit 2：基金、规则、告警等基础表结构与仓储

**Goal:**  
完成核心数据对象的表结构和仓储访问，为后续页面、采集和告警逻辑提供稳定数据层。

**Files:**  
- `modules/fund-monitor/migrations/`
- `modules/fund-monitor/src/domain/fund.rs`
- `modules/fund-monitor/src/domain/fund_quote.rs`
- `modules/fund-monitor/src/domain/monitor_rule.rs`
- `modules/fund-monitor/src/domain/alert_event.rs`
- `modules/fund-monitor/src/domain/job_run.rs`
- `modules/fund-monitor/src/domain/app_setting.rs`
- `modules/fund-monitor/src/storage/fund_repo.rs`
- `modules/fund-monitor/src/storage/rule_repo.rs`
- `modules/fund-monitor/src/storage/alert_repo.rs`
- `modules/fund-monitor/src/storage/job_repo.rs`
- `modules/fund-monitor/tests/storage_repositories.rs`

**Approach:**  
- 先按计划文档中的核心对象定义 MVP 最小字段。  
- 仓储接口先满足 CRUD、列表、最近数据查询和状态更新。  
- 历史表与当前状态都保留，避免后续页面无法回看历史。  
- 规则配置先用简单字段或 JSON 字段承载，避免首期过早抽象。

**Execution note:**  
Start with repository-level integration tests for insert, query, update, and status transitions.

**Test scenarios:**  
- 创建基金后，可按代码和名称查询到记录。  
- 删除或停用基金后，列表查询不再返回已停用项。  
- 写入多条 `fund_quote` 后，最近一条查询返回最新抓取结果。  
- 规则启用 / 停用后，启用规则查询只返回有效规则。  
- 告警状态从“新告警”更新为“已处理”后，状态查询正确反映变更。  
- 任务执行记录写入失败信息后，可按时间倒序查询。

**Verification:**  
- 数据层已能支撑基金、规则、告警、任务记录的基础访问。  
- 迁移和仓储测试能覆盖首期核心数据路径。

### Unit 3：基金池管理页面与基础 Web 路由

**Goal:**  
完成基金列表页、基金新增 / 编辑 / 删除入口和基础页面壳。

**Files:**  
- `modules/fund-monitor/src/web/mod.rs`
- `modules/fund-monitor/src/web/routes.rs`
- `modules/fund-monitor/src/web/funds.rs`
- `modules/fund-monitor/src/web/layout.rs`
- `modules/fund-monitor/templates/`
- `modules/fund-monitor/web/app.css`
- `modules/fund-monitor/web/app.js`
- `modules/fund-monitor/tests/fund_web_routes.rs`

**Approach:**  
- 页面渲染优先引入 `askama`，避免后续 HTML 字符串拼接失控。  
- 列表页先支持基础表格、分组展示和操作入口。  
- 表单提交先用标准 HTML 表单，避免过早引入复杂前端框架。  
- 保持页面和静态资源可继续由 `rust-embed` 提供。

**Execution note:**  
Implement request/response behavior test-first for fund list, create, update, and delete routes.

**Test scenarios:**  
- 访问 `/funds` 时，在无数据情况下返回空列表状态。  
- 提交合法基金代码和名称后，基金列表中出现新记录。  
- 提交重复基金代码时，页面返回明确错误提示。  
- 编辑备注和分组后，详情页和列表页都显示最新值。  
- 删除基金后，列表页不再显示该基金。  
- 非法表单输入时，不写入数据库且返回错误提示。

**Verification:**  
- 用户可通过页面完成基金池基础管理。  
- 页面结构已形成后续仪表盘、规则页和告警页可复用的布局壳。

### Unit 4：基金数据源接入与手动抓取链路

**Goal:**  
接入首个基金数据源，完成基金基础信息与行情数据的抓取、解析和入库。

**Files:**  
- `modules/fund-monitor/src/providers/mod.rs`
- `modules/fund-monitor/src/providers/fund_source.rs`
- `modules/fund-monitor/src/providers/http_client.rs`
- `modules/fund-monitor/src/app/errors.rs`
- `modules/fund-monitor/src/storage/fund_repo.rs`
- `modules/fund-monitor/src/storage/job_repo.rs`
- `modules/fund-monitor/src/web/funds.rs`
- `modules/fund-monitor/tests/provider_ingest.rs`

**Approach:**  
- 先抽象单一数据源接口，避免业务层直接依赖外部接口格式。  
- 首期实现“手动抓取单基金”能力，先打通一条从页面触发到入库的完整链路。  
- 将抓取结果写入 `fund_quote`，同时记录 `job_run`。  
- 抓取失败要保留错误信息，供页面和任务系统查看。

**Execution note:**  
Add characterization-style tests around provider response parsing before wiring it into handlers.

**Test scenarios:**  
- 数据源返回合法基金数据时，系统写入 `fund_quote` 并记录成功任务。  
- 数据源返回缺失字段时，系统拒绝入库并记录解析错误。  
- 网络请求失败时，系统写入失败任务记录，不产生脏数据。  
- 从页面触发单基金抓取后，基金详情页展示最近一次抓取结果。  
- 同一基金连续抓取两次后，最近数据查询返回最新记录。

**Verification:**  
- 首个基金数据源已经接入并可稳定写入历史数据。  
- 手动抓取链路打通，后续只需复用到定时任务。

### Unit 5：定时轮询任务与任务执行记录

**Goal:**  
完成基金轮询调度器，支持按固定频率批量抓取基金并生成任务执行记录。

**Files:**  
- `modules/fund-monitor/src/jobs/mod.rs`
- `modules/fund-monitor/src/jobs/scheduler.rs`
- `modules/fund-monitor/src/jobs/poll_funds.rs`
- `modules/fund-monitor/src/storage/job_repo.rs`
- `modules/fund-monitor/src/storage/fund_repo.rs`
- `modules/fund-monitor/src/app/config.rs`
- `modules/fund-monitor/tests/job_scheduler.rs`

**Approach:**  
- 首期在应用进程内运行单实例调度器。  
- 调度器根据配置频率轮询启用基金。  
- 每轮执行前后记录 `job_run`，失败时写入错误原因。  
- 为后续规则判断和告警链路预留调度后的扩展点。

**Test scenarios:**  
- 配置轮询频率后，调度器按预期触发抓取任务。  
- 基金列表为空时，任务跳过抓取但仍记录任务执行。  
- 单只基金抓取失败时，不影响本轮其他基金继续抓取。  
- 一轮执行结束后，任务记录正确包含开始、结束和结果状态。  
- 禁用基金不会被调度器选入抓取列表。

**Verification:**  
- 系统已具备后台自动轮询能力。  
- 后续规则判断可直接挂接在轮询完成后的数据流上。

### Unit 6：监控规则执行与告警事件生成

**Goal:**  
实现规则判断引擎，支持按基金最新数据生成告警事件并抑制短时间重复告警。

**Files:**  
- `modules/fund-monitor/src/domain/rule_engine.rs`
- `modules/fund-monitor/src/domain/monitor_rule.rs`
- `modules/fund-monitor/src/storage/rule_repo.rs`
- `modules/fund-monitor/src/storage/alert_repo.rs`
- `modules/fund-monitor/src/jobs/poll_funds.rs`
- `modules/fund-monitor/tests/rule_engine.rs`

**Approach:**  
- 首期支持三类规则：涨跌幅、净值区间、估值偏离。  
- 规则执行只基于最新基金数据，不在首期加入复杂趋势计算。  
- 冷却时间通过最近告警记录控制，避免重复推送。  
- 规则判断结果写入 `alert_event`，并返回给通知层。

**Execution note:**  
Implement new domain behavior test-first.

**Test scenarios:**  
- 涨跌幅超过阈值时，生成一条新告警。  
- 涨跌幅未超过阈值时，不生成告警。  
- 净值落入指定区间时，生成对应类型告警。  
- 同一规则在冷却时间内再次命中时，不重复生成新告警。  
- 已停用规则不会参与规则执行。  
- 多条规则同时命中同一基金时，分别生成独立告警事件。

**Verification:**  
- 规则引擎可稳定输出可追踪的告警事件。  
- 告警抑制逻辑可避免明显重复噪音。

### Unit 7：告警页面与 Telegram 通知

**Goal:**  
完成告警列表、告警状态处理和首个外部通知渠道 Telegram Bot。

**Files:**  
- `modules/fund-monitor/src/notifications/mod.rs`
- `modules/fund-monitor/src/notifications/telegram.rs`
- `modules/fund-monitor/src/web/alerts.rs`
- `modules/fund-monitor/src/storage/alert_repo.rs`
- `modules/fund-monitor/src/app/config.rs`
- `modules/fund-monitor/templates/`
- `modules/fund-monitor/tests/telegram_notifications.rs`

**Approach:**  
- 告警事件生成后先保证站内可查看，再补 Telegram 推送。  
- Telegram 配置通过环境变量或配置文件注入。  
- 推送失败不回滚告警事件，但要记录失败原因。  
- 页面支持“已处理 / 已忽略”状态变更。

**Test scenarios:**  
- 新告警生成后，告警列表页能看到该事件。  
- 将告警标记为“已处理”后，状态在列表页正确展示。  
- Telegram 配置完整且发送成功时，记录成功推送结果。  
- Telegram 配置缺失时，不发送外部通知并返回明确错误。  
- Telegram 请求失败时，告警事件保留，失败信息可追踪。

**Verification:**  
- 告警已具备站内查看和首个外部通知渠道。  
- 告警状态处理闭环可用。

### Unit 8：总览页、历史数据页与基础收口

**Goal:**  
补齐总览看板、基金历史数据展示、基础错误处理和日志收口。

**Files:**  
- `modules/fund-monitor/src/web/dashboard.rs`
- `modules/fund-monitor/src/web/funds.rs`
- `modules/fund-monitor/src/web/settings.rs`
- `modules/fund-monitor/src/app/errors.rs`
- `modules/fund-monitor/src/app/logging.rs`
- `modules/fund-monitor/templates/`
- `modules/fund-monitor/web/app.css`
- `modules/fund-monitor/tests/dashboard_routes.rs`

**Approach:**  
- 总览页优先展示抓取状态、基金数量、最新告警摘要。  
- 基金详情页补充历史数据列表，首期先不做复杂图表。  
- 统一基础错误页和日志记录格式，便于后续排查问题。  
- 设置页先覆盖轮询频率、数据源和通知渠道的基础配置展示。

**Test scenarios:**  
- 访问 `/dashboard` 时，可看到今日概览和最新告警摘要。  
- 基金详情页有历史数据时，按时间倒序展示最近记录。  
- 无历史数据时，页面展示空状态而不是错误。  
- 配置缺失或内部错误时，页面返回统一错误响应。  
- 日志中能区分抓取失败、规则失败和通知失败三类错误。

**Verification:**  
- 页面层完成首期看板闭环。  
- 系统具备基础可观测性和错误收口能力。

## 12. 建议执行顺序

建议按以下依赖顺序实施：

1. Unit 1：应用骨架、配置与数据库初始化
2. Unit 2：基础表结构与仓储
3. Unit 3：基金池管理页面
4. Unit 4：数据源接入与手动抓取
5. Unit 5：定时轮询任务
6. Unit 6：规则执行与告警事件
7. Unit 7：告警页面与 Telegram 通知
8. Unit 8：总览页、历史数据页与基础收口

其中可并行的部分：
- Unit 3 与 Unit 4 在 Unit 2 完成后可部分并行
- Unit 7 与 Unit 8 在 Unit 6 完成后可部分并行
