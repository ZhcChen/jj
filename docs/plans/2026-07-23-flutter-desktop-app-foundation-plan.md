# Flutter 桌面端 app 模块基础方案

日期：2026-07-23

## 1. 目标

在多模块仓库中新增 `modules/app`，作为基金监控项目的 Flutter 桌面端模块。

当前阶段目标：

- 先支持 `Windows / macOS / Linux`
- 将当前 Web 模块的信息结构迁移到桌面端
- 先建立桌面端 UI 壳、页面结构与发布链路
- 用 GitHub Actions 完成桌面构建与 GitHub Release 产物整理

未来阶段：

- 补 `Android / iOS`
- 将桌面端从种子数据切换到真实基金监控数据

## 2. 模块位置与命名

- 模块目录：`modules/app`
- 当前桌面实现技术：`Flutter`
- 当前模块用途：桌面端应用

说明：

- 仓库仍按 `modules/` 组织
- `fund-monitor` 继续承载 Rust 后端与当前 Web 应用
- `app` 负责桌面端交互层和桌面分发链路

## 3. 当前阶段范围

### 3.1 纳入本次

- 生成 Flutter 桌面端模块骨架
- 仅生成 `windows / macos / linux` 平台工程
- 搭建桌面端主工作区
- 映射当前页面结构：
  - 总览看板
  - 基金列表
  - 规则管理
  - 告警列表
  - 系统配置
- 将基金详情页按“只读 + 基金资料与行情快照融合”的方向迁移
- 增加 GitHub Actions 发版工作流
- 整理 GitHub Release 下载产物

### 3.2 暂不纳入本次

- Android / iOS 工程生成
- 真实 API / 本地数据库接入
- 桌面端系统托盘、通知、自动启动
- 原生签名 / notarization / Windows 代码签名

## 4. 技术策略

### 4.1 Flutter 模块生成

参考 Flutter 官方桌面支持说明：

- 当前用 Flutter 独立生成桌面三端工程
- 未来再通过 `flutter create --platforms=android,ios .` 补移动端

### 4.2 迁移策略

当前不直接硬连 `fund-monitor` 的 Rust Web 模块，而是先：

1. 对齐页面信息架构
2. 对齐视觉层级
3. 对齐主要对象模型
4. 用种子数据完成桌面端首轮壳验证

后续真实接入可选路径：

- 路径 A：Flutter 请求 Rust 暴露的本地 HTTP / JSON API
- 路径 B：桌面端与 Rust 共享本地 SQLite
- 路径 C：新增 Rust 桌面桥接层，由 Flutter 通过平台通道访问

当前建议优先保留 A / C 两条路线，后续根据部署方式再定。

## 5. GitHub Actions 发布策略

参考 `~/code/App-Manager` 的 release 组织方式，当前仓库采用：

- tag 驱动发版
- 多平台独立构建
- 产物统一汇总到 GitHub Release

为避免 monorepo 内不同模块 tag 混淆，桌面端采用：

- tag 格式：`app-vX.Y.Z`

发布流程：

1. 更新 `modules/app/pubspec.yaml` 版本，如 `0.1.0+1`
2. 合并到 `main`
3. 推送 tag：`app-v0.1.0`
4. 工作流自动：
   - 校验 tag 与 `pubspec.yaml` 版本一致
   - 在 Windows / macOS / Linux 构建 release
   - 将可下载内容整理为 zip 资产
   - 创建或更新 GitHub Release

当前发布资产命名规则：

- `fund-monitor-app-<version>-<platform>-<arch>.zip`
- 例如：
  - `fund-monitor-app-0.1.0-macos-arm64.zip`
  - `fund-monitor-app-0.1.0-windows-x64.zip`
  - `fund-monitor-app-0.1.0-linux-x64.zip`

## 6. 目录建议

```text
modules/app/
├── lib/
│   ├── main.dart
│   └── src/
├── test/
├── tool/
│   ├── verify_release_tag.dart
│   └── generate_release_notes.dart
├── windows/
├── macos/
└── linux/
```

## 7. 分阶段实施

### Phase 1：桌面端基础模块

- 生成 Flutter 模块
- 建立主题、导航、页面壳
- 建立桌面端只读详情页
- 提供种子数据

验收：

- `flutter analyze` 通过
- `flutter test` 通过
- 本机可运行桌面端

### Phase 2：GitHub Release 自动化

- 增加 tag 校验脚本
- 增加 release notes 生成脚本
- 增加多平台 release workflow
- 统一下载资产命名

验收：

- 通过 `app-v*.*.*` tag 可以自动创建 release
- release 页面下载资产清晰、按平台区分

### Phase 3：真实数据接入

- 对接 `fund-monitor` 真实数据
- 替换桌面端种子数据
- 增加详情自动刷新、规则与告警的真实状态

## 8. 当前实现边界

当前落地应明确标注：

- 这是桌面端基础模块
- 当前重心是壳、结构和发版链路
- 当前不是完整功能等价迁移

这样可以保证：

- 先把桌面端模块在仓库里站稳
- 先把 CI / Release 跑通
- 再渐进迁移真实业务能力
