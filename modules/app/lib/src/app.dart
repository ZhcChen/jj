import 'package:flutter/material.dart';

import 'app_data.dart';
import 'app_theme.dart';

const String _brandMarkAsset = 'brand/logo/fund-monitor-mark.png';

void runFundMonitorDesktopApp() {
  runApp(const FundMonitorDesktopApp());
}

class FundMonitorDesktopApp extends StatelessWidget {
  const FundMonitorDesktopApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: '基金监控桌面端',
      debugShowCheckedModeBanner: false,
      theme: buildAppTheme(),
      home: const DesktopWorkspace(),
    );
  }
}

class DesktopWorkspace extends StatefulWidget {
  const DesktopWorkspace({super.key});

  @override
  State<DesktopWorkspace> createState() => _DesktopWorkspaceState();
}

class _DesktopWorkspaceState extends State<DesktopWorkspace> {
  final DesktopSeedData _data = buildDesktopSeedData();
  DesktopSection _section = DesktopSection.dashboard;
  int _selectedFundIndex = 0;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.fromLTRB(16, 14, 16, 18),
          child: Column(
            children: [
              _HeaderBar(
                section: _section,
                lastRefreshAt: _data.lastRefreshAt,
                onSectionChanged: (section) {
                  setState(() {
                    _section = section;
                  });
                },
              ),
              const SizedBox(height: 14),
              Expanded(
                child: DecoratedBox(
                  decoration: BoxDecoration(
                    borderRadius: BorderRadius.circular(28),
                    boxShadow: const [
                      BoxShadow(
                        color: AppPalette.shadow,
                        blurRadius: 42,
                        offset: Offset(0, 18),
                      ),
                    ],
                  ),
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(28),
                    child: ColoredBox(
                      color: const Color(0xCCFFFFFF),
                      child: SingleChildScrollView(
                        padding: const EdgeInsets.all(18),
                        child: AnimatedSwitcher(
                          duration: const Duration(milliseconds: 180),
                          child: KeyedSubtree(
                            key: ValueKey<DesktopSection>(_section),
                            child: _buildSection(theme),
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildSection(ThemeData theme) {
    switch (_section) {
      case DesktopSection.dashboard:
        return DashboardSection(data: _data);
      case DesktopSection.funds:
        return FundsSection(
          data: _data,
          selectedIndex: _selectedFundIndex,
          onFundSelected: (index) {
            setState(() {
              _selectedFundIndex = index;
            });
          },
        );
      case DesktopSection.rules:
        return RulesSection(data: _data);
      case DesktopSection.alerts:
        return AlertsSection(data: _data);
      case DesktopSection.settings:
        return SettingsSection(data: _data);
    }
  }
}

class _HeaderBar extends StatelessWidget {
  const _HeaderBar({
    required this.section,
    required this.lastRefreshAt,
    required this.onSectionChanged,
  });

  final DesktopSection section;
  final String lastRefreshAt;
  final ValueChanged<DesktopSection> onSectionChanged;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Card(
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 12),
        child: LayoutBuilder(
          builder: (context, constraints) {
            final isWideLayout = constraints.maxWidth >= 1280;

            if (isWideLayout) {
              return SizedBox(
                height: 72,
                child: Stack(
                  alignment: Alignment.center,
                  children: [
                    Align(
                      alignment: Alignment.centerLeft,
                      child: _HeaderBrand(theme: theme),
                    ),
                    Align(
                      alignment: Alignment.center,
                      child: Wrap(
                        alignment: WrapAlignment.center,
                        spacing: 10,
                        runSpacing: 10,
                        children: [
                          for (final item in DesktopSection.values)
                            _NavChip(
                              item: item,
                              selected: item == section,
                              onTap: () => onSectionChanged(item),
                            ),
                        ],
                      ),
                    ),
                    Align(
                      alignment: Alignment.centerRight,
                      child: _HeaderRefresh(lastRefreshAt: lastRefreshAt),
                    ),
                  ],
                ),
              );
            }

            return Column(
              children: [
                Row(
                  children: [
                    Expanded(child: _HeaderBrand(theme: theme)),
                    const SizedBox(width: 12),
                    _HeaderRefresh(lastRefreshAt: lastRefreshAt),
                  ],
                ),
                const SizedBox(height: 16),
                Wrap(
                  alignment: WrapAlignment.center,
                  spacing: 10,
                  runSpacing: 10,
                  children: [
                    for (final item in DesktopSection.values)
                      _NavChip(
                        item: item,
                        selected: item == section,
                        onTap: () => onSectionChanged(item),
                      ),
                  ],
                ),
              ],
            );
          },
        ),
      ),
    );
  }
}

class _HeaderBrand extends StatelessWidget {
  const _HeaderBrand({required this.theme});

  final ThemeData theme;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        SizedBox(
          width: 44,
          height: 44,
          child: Image.asset(
            _brandMarkAsset,
            fit: BoxFit.contain,
            filterQuality: FilterQuality.high,
          ),
        ),
        const SizedBox(width: 12),
        Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('基金监控桌面端', style: theme.textTheme.titleLarge),
            const SizedBox(height: 3),
            Text('专业基金监控终端 · Flutter 桌面版', style: theme.textTheme.bodySmall),
          ],
        ),
      ],
    );
  }
}

class _HeaderRefresh extends StatelessWidget {
  const _HeaderRefresh({required this.lastRefreshAt});

  final String lastRefreshAt;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: AppPalette.panelMuted,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          Text('最近刷新', style: theme.textTheme.labelMedium),
          const SizedBox(height: 2),
          Text(lastRefreshAt, style: theme.textTheme.titleMedium),
        ],
      ),
    );
  }
}

class _NavChip extends StatelessWidget {
  const _NavChip({
    required this.item,
    required this.selected,
    required this.onTap,
  });

  final DesktopSection item;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(999),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 160),
        padding: const EdgeInsets.symmetric(horizontal: 13, vertical: 8),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(999),
          color: selected ? AppPalette.accent : AppPalette.panelMuted,
          boxShadow: selected
              ? const [
                  BoxShadow(
                    color: Color(0x262F6FD6),
                    blurRadius: 18,
                    offset: Offset(0, 8),
                  ),
                ]
              : null,
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              item.icon,
              size: 17,
              color: selected ? Colors.white : AppPalette.textSoft,
            ),
            const SizedBox(width: 8),
            Text(
              item.label,
              style: TextStyle(
                color: selected ? Colors.white : AppPalette.text,
                fontSize: 13,
                fontWeight: FontWeight.w700,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class DashboardSection extends StatelessWidget {
  const DashboardSection({super.key, required this.data});

  final DesktopSeedData data;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('总览看板', style: Theme.of(context).textTheme.headlineMedium),
        const SizedBox(height: 8),
        Text(
          '桌面端先对齐现有信息架构与视觉层级，下一步再把 fund-monitor 的真实数据链路接过来。',
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        const SizedBox(height: 18),
        _MetricGrid(metrics: data.metrics),
        const SizedBox(height: 18),
        LayoutBuilder(
          builder: (context, constraints) {
            final twoColumns = constraints.maxWidth >= 1120;
            return Flex(
              direction: twoColumns ? Axis.horizontal : Axis.vertical,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(
                  flex: twoColumns ? 8 : 0,
                  child: _InfoCard(
                    title: '桌面迁移范围',
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: const [
                        _BulletRow('新增 `modules/app` Flutter 桌面模块'),
                        _BulletRow('当前先支持 Windows / macOS / Linux'),
                        _BulletRow('桌面 UI 已映射总览、基金、规则、告警、设置五个工作区'),
                        _BulletRow('后续可继续补 Android / iOS 与真实数据接入'),
                      ],
                    ),
                  ),
                ),
                SizedBox(
                  width: twoColumns ? 16 : 0,
                  height: twoColumns ? 0 : 16,
                ),
                Expanded(
                  flex: twoColumns ? 6 : 0,
                  child: _InfoCard(
                    title: '默认基金',
                    trailing: const _Badge('桌面基准对象'),
                    child: _FundCompactCard(fund: data.funds.first),
                  ),
                ),
              ],
            );
          },
        ),
        const SizedBox(height: 18),
        LayoutBuilder(
          builder: (context, constraints) {
            final twoColumns = constraints.maxWidth >= 1120;
            return Flex(
              direction: twoColumns ? Axis.horizontal : Axis.vertical,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(
                  flex: twoColumns ? 7 : 0,
                  child: _InfoCard(
                    title: '最近任务',
                    child: Column(
                      children: [
                        for (final job in data.jobs) ...[
                          _JobRow(job: job),
                          if (job != data.jobs.last) const SizedBox(height: 10),
                        ],
                      ],
                    ),
                  ),
                ),
                SizedBox(
                  width: twoColumns ? 16 : 0,
                  height: twoColumns ? 0 : 16,
                ),
                Expanded(
                  flex: twoColumns ? 7 : 0,
                  child: _InfoCard(
                    title: '最新告警',
                    child: Column(
                      children: [
                        for (final alert in data.alerts) ...[
                          _AlertRow(alert: alert),
                          if (alert != data.alerts.last)
                            const SizedBox(height: 10),
                        ],
                      ],
                    ),
                  ),
                ),
              ],
            );
          },
        ),
      ],
    );
  }
}

class FundsSection extends StatelessWidget {
  const FundsSection({
    super.key,
    required this.data,
    required this.selectedIndex,
    required this.onFundSelected,
  });

  final DesktopSeedData data;
  final int selectedIndex;
  final ValueChanged<int> onFundSelected;

  @override
  Widget build(BuildContext context) {
    final selectedFund = data.funds[selectedIndex];

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('基金列表', style: Theme.of(context).textTheme.headlineMedium),
        const SizedBox(height: 8),
        Text(
          '桌面端详情页已改为只读，并把基金资料和行情快照融合到同一张主卡片。',
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        const SizedBox(height: 18),
        LayoutBuilder(
          builder: (context, constraints) {
            final twoColumns = constraints.maxWidth >= 1180;
            return Flex(
              direction: twoColumns ? Axis.horizontal : Axis.vertical,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                SizedBox(
                  width: twoColumns ? 360 : double.infinity,
                  child: _InfoCard(
                    title: '基金池',
                    trailing: _Badge('${data.funds.length} 只'),
                    child: Column(
                      children: [
                        for (
                          var index = 0;
                          index < data.funds.length;
                          index++
                        ) ...[
                          _FundListRow(
                            fund: data.funds[index],
                            selected: index == selectedIndex,
                            onTap: () => onFundSelected(index),
                          ),
                          if (index != data.funds.length - 1)
                            const SizedBox(height: 10),
                        ],
                      ],
                    ),
                  ),
                ),
                SizedBox(
                  width: twoColumns ? 16 : 0,
                  height: twoColumns ? 0 : 16,
                ),
                if (twoColumns)
                  Expanded(child: FundDetailReadonlyCard(fund: selectedFund))
                else
                  FundDetailReadonlyCard(fund: selectedFund),
              ],
            );
          },
        ),
      ],
    );
  }
}

class RulesSection extends StatelessWidget {
  const RulesSection({super.key, required this.data});

  final DesktopSeedData data;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('规则管理', style: Theme.of(context).textTheme.headlineMedium),
        const SizedBox(height: 8),
        Text(
          '规则页先承接现有 Web 模块的规则语义，后续再补桌面端编辑与启停动作。',
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        const SizedBox(height: 18),
        for (final rule in data.rules) ...[
          _InfoCard(
            title: rule.name,
            trailing: _Badge(rule.status),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _KeyValueText(label: '作用范围', value: rule.scope),
                const SizedBox(height: 10),
                _KeyValueText(label: '规则条件', value: rule.condition),
                const SizedBox(height: 10),
                _KeyValueText(label: '冷却时间', value: rule.cooldown),
              ],
            ),
          ),
          if (rule != data.rules.last) const SizedBox(height: 14),
        ],
      ],
    );
  }
}

class AlertsSection extends StatelessWidget {
  const AlertsSection({super.key, required this.data});

  final DesktopSeedData data;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('告警列表', style: Theme.of(context).textTheme.headlineMedium),
        const SizedBox(height: 8),
        Text(
          '这里先展示桌面端的告警阅读模型，后续可再接入告警状态处理和外部通知联动。',
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        const SizedBox(height: 18),
        for (final alert in data.alerts) ...[
          _InfoCard(
            title: alert.title,
            trailing: _Badge('等级 ${alert.level}'),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  alert.summary,
                  style: Theme.of(context).textTheme.bodyLarge,
                ),
                const SizedBox(height: 12),
                _KeyValueText(label: '触发时间', value: alert.occurredAt),
              ],
            ),
          ),
          if (alert != data.alerts.last) const SizedBox(height: 14),
        ],
      ],
    );
  }
}

class SettingsSection extends StatelessWidget {
  const SettingsSection({super.key, required this.data});

  final DesktopSeedData data;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('系统配置', style: Theme.of(context).textTheme.headlineMedium),
        const SizedBox(height: 8),
        Text(
          '桌面端首期重点是模块骨架、信息结构和 GitHub Release 构建链路。',
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        const SizedBox(height: 18),
        for (final group in data.settings) ...[
          _InfoCard(
            title: group.title,
            child: Column(
              children: [
                for (final item in group.items) ...[
                  _SettingRow(item: item),
                  if (item != group.items.last) const SizedBox(height: 12),
                ],
              ],
            ),
          ),
          if (group != data.settings.last) const SizedBox(height: 14),
        ],
      ],
    );
  }
}

class FundDetailReadonlyCard extends StatelessWidget {
  const FundDetailReadonlyCard({super.key, required this.fund});

  final FundRecord fund;

  @override
  Widget build(BuildContext context) {
    final snapshot = fund.snapshot;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(18),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: const [
                Expanded(child: _SectionTitle('基金详情')),
                _Badge('只读'),
              ],
            ),
            const SizedBox(height: 16),
            Container(
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: AppPalette.panelMuted,
                borderRadius: BorderRadius.circular(18),
              ),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              fund.name,
                              style: Theme.of(context).textTheme.headlineMedium,
                            ),
                            const SizedBox(height: 6),
                            Text(
                              fund.code,
                              style: const TextStyle(
                                color: AppPalette.accent,
                                fontSize: 13,
                                fontWeight: FontWeight.w700,
                                letterSpacing: 0.24,
                              ),
                            ),
                          ],
                        ),
                      ),
                      _Badge(snapshot == null ? '暂无快照' : '已接入快照'),
                    ],
                  ),
                  const SizedBox(height: 16),
                  _DetailOverviewGrid(fund: fund),
                ],
              ),
            ),
            const SizedBox(height: 16),
            if (snapshot == null)
              const _EmptySnapshot()
            else
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const _SubHeader(title: '行情快照', badge: '自动刷新预留'),
                  const SizedBox(height: 12),
                  const _SubHeader(title: '确认净值口径', badge: '官方净值'),
                  const SizedBox(height: 12),
                  _InfoTileGrid(
                    children: [
                      _MetricTile(label: '净值日期', value: snapshot.navDate),
                      _MetricTile(label: '确认净值', value: snapshot.unitNav),
                      _MetricTile(
                        label: '净值日涨跌幅',
                        value: snapshot.confirmedChangeRate,
                        tone: snapshot.confirmedTone,
                      ),
                    ],
                  ),
                  const SizedBox(height: 12),
                  _SubHeader(
                    title: '盘中估值口径',
                    badge: snapshot.hasEstimatedSnapshot ? '实时估算' : '暂不可用',
                    highlighted: snapshot.hasEstimatedSnapshot,
                  ),
                  const SizedBox(height: 12),
                  _InfoTileGrid(
                    children: [
                      _MetricTile(label: '估值', value: snapshot.estimatedNav),
                      _MetricTile(
                        label: '估算涨跌幅',
                        value: snapshot.estimatedChangeRate,
                        tone: snapshot.estimatedTone,
                      ),
                      _MetricTile(label: '估值时间', value: snapshot.estimatedAt),
                    ],
                  ),
                  if (!snapshot.hasEstimatedSnapshot) ...[
                    const SizedBox(height: 12),
                    Text(
                      '当前基金暂未返回盘中估值字段，因此这里只展示确认净值口径。',
                      style: Theme.of(context).textTheme.bodyMedium,
                    ),
                  ],
                  const SizedBox(height: 12),
                  _InfoTileGrid(
                    children: [
                      _MetaTile(label: '抓取时间', value: snapshot.fetchedAt),
                      _MetaTile(label: '数据源', value: snapshot.source),
                    ],
                  ),
                ],
              ),
          ],
        ),
      ),
    );
  }
}

class _MetricGrid extends StatelessWidget {
  const _MetricGrid({required this.metrics});

  final List<MetricCardData> metrics;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final maxWidth = constraints.maxWidth;
        final crossAxisCount = maxWidth >= 1240
            ? 4
            : maxWidth >= 940
            ? 3
            : maxWidth >= 620
            ? 2
            : 1;

        return GridView.builder(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          itemCount: metrics.length,
          gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
            crossAxisCount: crossAxisCount,
            crossAxisSpacing: 12,
            mainAxisSpacing: 12,
            mainAxisExtent: 126,
          ),
          itemBuilder: (context, index) {
            final metric = metrics[index];
            return Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      metric.label,
                      style: Theme.of(context).textTheme.labelMedium,
                    ),
                    const Spacer(),
                    Text(
                      metric.value,
                      style: Theme.of(context).textTheme.headlineMedium,
                    ),
                    const SizedBox(height: 8),
                    Text(
                      metric.hint,
                      style: Theme.of(context).textTheme.bodyMedium,
                    ),
                  ],
                ),
              ),
            );
          },
        );
      },
    );
  }
}

class _InfoCard extends StatelessWidget {
  const _InfoCard({required this.title, required this.child, this.trailing});

  final String title;
  final Widget child;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    final headerChildren = <Widget>[Expanded(child: _SectionTitle(title))];
    if (trailing != null) {
      headerChildren.add(trailing!);
    }

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(18),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(children: headerChildren),
            const SizedBox(height: 14),
            child,
          ],
        ),
      ),
    );
  }
}

class _SectionTitle extends StatelessWidget {
  const _SectionTitle(this.title);

  final String title;

  @override
  Widget build(BuildContext context) {
    return Text(title, style: Theme.of(context).textTheme.titleLarge);
  }
}

class _Badge extends StatelessWidget {
  const _Badge(this.text);

  final String text;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 7),
      decoration: BoxDecoration(
        color: AppPalette.accentSoft,
        borderRadius: BorderRadius.circular(999),
      ),
      child: Text(
        text,
        style: const TextStyle(
          color: AppPalette.accent,
          fontSize: 12,
          fontWeight: FontWeight.w700,
        ),
      ),
    );
  }
}

class _BulletRow extends StatelessWidget {
  const _BulletRow(this.text);

  final String text;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 10),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: 7,
            height: 7,
            margin: const EdgeInsets.only(top: 7),
            decoration: const BoxDecoration(
              color: AppPalette.accent,
              shape: BoxShape.circle,
            ),
          ),
          const SizedBox(width: 10),
          Expanded(
            child: Text(text, style: Theme.of(context).textTheme.bodyLarge),
          ),
        ],
      ),
    );
  }
}

class _FundCompactCard extends StatelessWidget {
  const _FundCompactCard({required this.fund});

  final FundRecord fund;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(fund.name, style: Theme.of(context).textTheme.titleLarge),
        const SizedBox(height: 6),
        Text(
          fund.code,
          style: const TextStyle(
            color: AppPalette.accent,
            fontSize: 13,
            fontWeight: FontWeight.w700,
          ),
        ),
        const SizedBox(height: 12),
        Text(fund.note, style: Theme.of(context).textTheme.bodyMedium),
      ],
    );
  }
}

class _FundListRow extends StatelessWidget {
  const _FundListRow({
    required this.fund,
    required this.selected,
    required this.onTap,
  });

  final FundRecord fund;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      borderRadius: BorderRadius.circular(18),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 160),
        padding: const EdgeInsets.all(14),
        decoration: BoxDecoration(
          color: selected ? AppPalette.accentSoft : AppPalette.panelMuted,
          borderRadius: BorderRadius.circular(18),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    fund.name,
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ),
                if (selected) const _Badge('当前查看'),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              fund.code,
              style: const TextStyle(
                color: AppPalette.accent,
                fontSize: 12,
                fontWeight: FontWeight.w700,
              ),
            ),
            const SizedBox(height: 8),
            Text(fund.groupName, style: Theme.of(context).textTheme.bodyMedium),
          ],
        ),
      ),
    );
  }
}

class _DetailInfoTile extends StatelessWidget {
  const _DetailInfoTile({
    required this.label,
    required this.value,
    this.muted = false,
  });

  final String label;
  final String value;
  final bool muted;

  @override
  Widget build(BuildContext context) {
    return Container(
      constraints: const BoxConstraints(minHeight: 92),
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: AppPalette.panel,
        borderRadius: BorderRadius.circular(16),
        boxShadow: const [
          BoxShadow(
            color: AppPalette.shadow,
            blurRadius: 20,
            offset: Offset(0, 8),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: Theme.of(context).textTheme.labelMedium),
          const SizedBox(height: 8),
          Text(
            value,
            style: Theme.of(context).textTheme.bodyLarge?.copyWith(
              color: muted ? AppPalette.textSoft : AppPalette.text,
              fontWeight: FontWeight.w700,
            ),
          ),
        ],
      ),
    );
  }
}

class _DetailOverviewGrid extends StatelessWidget {
  const _DetailOverviewGrid({required this.fund});

  final FundRecord fund;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final twoColumns = constraints.maxWidth >= 840;

        return Column(
          children: [
            if (twoColumns)
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(
                    child: _DetailInfoTile(label: '基金代码', value: fund.code),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: _DetailInfoTile(label: '基金名称', value: fund.name),
                  ),
                ],
              )
            else ...[
              _DetailInfoTile(label: '基金代码', value: fund.code),
              const SizedBox(height: 12),
              _DetailInfoTile(label: '基金名称', value: fund.name),
            ],
            const SizedBox(height: 12),
            if (twoColumns)
              Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Expanded(
                    child: _DetailInfoTile(
                      label: '分组',
                      value: fund.groupName.isEmpty ? '-' : fund.groupName,
                      muted: fund.groupName.isEmpty,
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: _DetailInfoTile(
                      label: '标签',
                      value: fund.tags.isEmpty ? '-' : fund.tags,
                      muted: fund.tags.isEmpty,
                    ),
                  ),
                ],
              )
            else ...[
              _DetailInfoTile(
                label: '分组',
                value: fund.groupName.isEmpty ? '-' : fund.groupName,
                muted: fund.groupName.isEmpty,
              ),
              const SizedBox(height: 12),
              _DetailInfoTile(
                label: '标签',
                value: fund.tags.isEmpty ? '-' : fund.tags,
                muted: fund.tags.isEmpty,
              ),
            ],
            const SizedBox(height: 12),
            _DetailInfoTile(
              label: '备注',
              value: fund.note.isEmpty ? '-' : fund.note,
              muted: fund.note.isEmpty,
            ),
          ],
        );
      },
    );
  }
}

class _MetricTile extends StatelessWidget {
  const _MetricTile({
    required this.label,
    required this.value,
    this.tone = ChangeTone.neutral,
  });

  final String label;
  final String value;
  final ChangeTone tone;

  @override
  Widget build(BuildContext context) {
    Color color = AppPalette.text;
    if (tone == ChangeTone.up) {
      color = AppPalette.up;
    } else if (tone == ChangeTone.down) {
      color = AppPalette.down;
    }

    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: AppPalette.panelMuted,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: Theme.of(context).textTheme.labelMedium),
          const SizedBox(height: 10),
          Text(
            value,
            style: Theme.of(
              context,
            ).textTheme.headlineMedium?.copyWith(color: color, fontSize: 20),
          ),
        ],
      ),
    );
  }
}

class _MetaTile extends StatelessWidget {
  const _MetaTile({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: AppPalette.panelMuted,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: Theme.of(context).textTheme.labelMedium),
          const SizedBox(height: 8),
          Text(value, style: Theme.of(context).textTheme.bodyLarge),
        ],
      ),
    );
  }
}

class _InfoTileGrid extends StatelessWidget {
  const _InfoTileGrid({required this.children});

  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final twoColumns = constraints.maxWidth >= 840;
        final crossAxisCount = twoColumns ? 2 : 1;

        return GridView.builder(
          shrinkWrap: true,
          physics: const NeverScrollableScrollPhysics(),
          itemCount: children.length,
          gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
            crossAxisCount: crossAxisCount,
            crossAxisSpacing: 12,
            mainAxisSpacing: 12,
            mainAxisExtent: 126,
          ),
          itemBuilder: (context, index) {
            return children[index];
          },
        );
      },
    );
  }
}

class _SubHeader extends StatelessWidget {
  const _SubHeader({
    required this.title,
    required this.badge,
    this.highlighted = false,
  });

  final String title;
  final String badge;
  final bool highlighted;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: Text(title, style: Theme.of(context).textTheme.titleMedium),
        ),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
          decoration: BoxDecoration(
            color: highlighted ? AppPalette.accentSoft : AppPalette.panelMuted,
            borderRadius: BorderRadius.circular(999),
          ),
          child: Text(
            badge,
            style: TextStyle(
              color: highlighted ? AppPalette.accent : AppPalette.textSoft,
              fontSize: 11,
              fontWeight: FontWeight.w700,
            ),
          ),
        ),
      ],
    );
  }
}

class _EmptySnapshot extends StatelessWidget {
  const _EmptySnapshot();

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: AppPalette.panelMuted,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Text('当前还没有任何抓取记录。', style: Theme.of(context).textTheme.bodyLarge),
    );
  }
}

class _KeyValueText extends StatelessWidget {
  const _KeyValueText({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: Theme.of(context).textTheme.labelMedium),
        const SizedBox(height: 4),
        Text(value, style: Theme.of(context).textTheme.bodyLarge),
      ],
    );
  }
}

class _SettingRow extends StatelessWidget {
  const _SettingRow({required this.item});

  final SettingItem item;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: AppPalette.panelMuted,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(item.label, style: Theme.of(context).textTheme.labelMedium),
          const SizedBox(height: 8),
          Text(item.value, style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 6),
          Text(item.hint, style: Theme.of(context).textTheme.bodyMedium),
        ],
      ),
    );
  }
}

class _JobRow extends StatelessWidget {
  const _JobRow({required this.job});

  final JobRecord job;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: AppPalette.panelMuted,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(job.name, style: Theme.of(context).textTheme.titleMedium),
                const SizedBox(height: 6),
                Text(job.note, style: Theme.of(context).textTheme.bodyMedium),
              ],
            ),
          ),
          const SizedBox(width: 14),
          Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: [
              _Badge(job.status),
              const SizedBox(height: 8),
              Text(job.startedAt, style: Theme.of(context).textTheme.bodySmall),
            ],
          ),
        ],
      ),
    );
  }
}

class _AlertRow extends StatelessWidget {
  const _AlertRow({required this.alert});

  final AlertRecord alert;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: AppPalette.panelMuted,
        borderRadius: BorderRadius.circular(16),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  alert.title,
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ),
              _Badge('等级 ${alert.level}'),
            ],
          ),
          const SizedBox(height: 8),
          Text(alert.summary, style: Theme.of(context).textTheme.bodyMedium),
          const SizedBox(height: 8),
          Text(alert.occurredAt, style: Theme.of(context).textTheme.bodySmall),
        ],
      ),
    );
  }
}
