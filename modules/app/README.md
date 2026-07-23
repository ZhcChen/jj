# app

`modules/app` 是基金监控项目的 Flutter 桌面端模块。

当前阶段目标：

- 用 Flutter 建立桌面端信息架构与视觉骨架
- 先支持 `Windows / macOS / Linux`
- 未来再补 `Android / iOS`
- 为 GitHub Actions 桌面构建与 GitHub Release 整理下载产物打基础

当前实现状态：

- 已完成桌面端主工作区壳
- 已映射总览、基金、规则、告警、设置五个页面
- 基金详情页已按“只读 + 基金资料与行情快照融合”的方向迁移
- 当前数据为桌面种子数据，后续再接 `modules/fund-monitor` 的真实数据链路

## 本地开发

```bash
cd modules/app
flutter pub get
flutter analyze
flutter test
flutter run -d macos
```

其他桌面端：

```bash
flutter run -d windows
flutter run -d linux
```

## 平台补充

当前模块只生成了桌面三端工程。未来如果要补移动端，可在 `modules/app` 下执行：

```bash
flutter create --platforms=android,ios .
```

## GitHub Release

桌面端发版采用 **tag 驱动**：

1. 更新 `modules/app/pubspec.yaml` 中的版本号，例如 `0.1.0+1`
2. 合并到 `main`
3. 推送 tag，例如 `app-v0.1.0`
4. GitHub Actions 工作流 `release-app.yml` 会自动：
   - 校验 tag 与 `pubspec.yaml` 版本一致
   - 在 Windows / macOS / Linux 构建桌面产物
   - 将各平台产物整理成 zip 下载包
   - 创建或更新对应的 GitHub Release
   - 生成按平台/架构分组的 release notes 与变更对比链接

当前发布产物命名统一为：

```text
fund-monitor-app-<version>-<platform>-<arch>.zip
```

命名由 `modules/app/tool/release_asset_name.dart` 统一生成。

示例：

- `fund-monitor-app-0.1.0-macos-arm64.zip`
- `fund-monitor-app-0.1.0-windows-x64.zip`
- `fund-monitor-app-0.1.0-linux-x64.zip`

如果已有 release 的说明需要重建，可使用：

- Actions 手动运行 `Repair App Release Notes`
- 或推送修复 tag，例如 `repair-app-release-notes/app-v0.1.0`
