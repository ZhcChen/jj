import 'package:flutter/material.dart';

enum DesktopSection {
  dashboard('总览看板', Icons.space_dashboard_rounded),
  funds('基金列表', Icons.account_balance_wallet_rounded),
  rules('规则管理', Icons.tune_rounded),
  alerts('告警列表', Icons.notifications_active_rounded),
  settings('系统配置', Icons.settings_rounded);

  const DesktopSection(this.label, this.icon);

  final String label;
  final IconData icon;
}

enum ChangeTone { neutral, up, down }

class MetricCardData {
  const MetricCardData({
    required this.label,
    required this.value,
    required this.hint,
  });

  final String label;
  final String value;
  final String hint;
}

class QuoteSnapshot {
  const QuoteSnapshot({
    required this.navDate,
    required this.unitNav,
    required this.confirmedChangeRate,
    required this.confirmedTone,
    required this.estimatedNav,
    required this.estimatedChangeRate,
    required this.estimatedTone,
    required this.estimatedAt,
    required this.fetchedAt,
    required this.source,
    required this.hasEstimatedSnapshot,
  });

  final String navDate;
  final String unitNav;
  final String confirmedChangeRate;
  final ChangeTone confirmedTone;
  final String estimatedNav;
  final String estimatedChangeRate;
  final ChangeTone estimatedTone;
  final String estimatedAt;
  final String fetchedAt;
  final String source;
  final bool hasEstimatedSnapshot;
}

class FundRecord {
  const FundRecord({
    required this.code,
    required this.name,
    required this.groupName,
    required this.tags,
    required this.note,
    required this.snapshot,
  });

  final String code;
  final String name;
  final String groupName;
  final String tags;
  final String note;
  final QuoteSnapshot? snapshot;
}

class RuleRecord {
  const RuleRecord({
    required this.name,
    required this.scope,
    required this.condition,
    required this.cooldown,
    required this.status,
  });

  final String name;
  final String scope;
  final String condition;
  final String cooldown;
  final String status;
}

class AlertRecord {
  const AlertRecord({
    required this.title,
    required this.summary,
    required this.level,
    required this.occurredAt,
  });

  final String title;
  final String summary;
  final String level;
  final String occurredAt;
}

class SettingGroup {
  const SettingGroup({required this.title, required this.items});

  final String title;
  final List<SettingItem> items;
}

class SettingItem {
  const SettingItem({
    required this.label,
    required this.value,
    required this.hint,
  });

  final String label;
  final String value;
  final String hint;
}

class JobRecord {
  const JobRecord({
    required this.name,
    required this.status,
    required this.startedAt,
    required this.note,
  });

  final String name;
  final String status;
  final String startedAt;
  final String note;
}

class DesktopSeedData {
  const DesktopSeedData({
    required this.metrics,
    required this.funds,
    required this.rules,
    required this.alerts,
    required this.settings,
    required this.jobs,
    required this.lastRefreshAt,
  });

  final List<MetricCardData> metrics;
  final List<FundRecord> funds;
  final List<RuleRecord> rules;
  final List<AlertRecord> alerts;
  final List<SettingGroup> settings;
  final List<JobRecord> jobs;
  final String lastRefreshAt;
}

DesktopSeedData buildDesktopSeedData({DateTime? now}) {
  final currentTime = _trimMilliseconds(now ?? DateTime.now());
  final latestNavDate = _formatDate(_previousTradingDay(currentTime));
  final primaryFetchTime = currentTime;
  final secondaryFetchTime = currentTime.subtract(const Duration(seconds: 47));
  final tertiaryFetchTime = currentTime.subtract(
    const Duration(minutes: 1, seconds: 13),
  );
  final primarySnapshot = QuoteSnapshot(
    navDate: latestNavDate,
    unitNav: '2.2499',
    confirmedChangeRate: '-1.26%',
    confirmedTone: ChangeTone.down,
    estimatedNav: '2.1883',
    estimatedChangeRate: '-2.74%',
    estimatedTone: ChangeTone.down,
    estimatedAt: _formatDateTime(primaryFetchTime),
    fetchedAt: _formatDateTime(primaryFetchTime),
    source: '东方财富净值快照 + 东方财富实时估值',
    hasEstimatedSnapshot: true,
  );

  return DesktopSeedData(
    metrics: const [
      MetricCardData(label: '启用基金', value: '3', hint: '当前桌面端展示的监控基金池规模'),
      MetricCardData(label: '轮询频率', value: '60 秒', hint: '与现有 Web 模块保持一致'),
      MetricCardData(label: '活跃规则', value: '4', hint: '已启用的监控规则数量'),
      MetricCardData(label: '待处理告警', value: '2', hint: '桌面端后续会提供快捷处理入口'),
    ],
    funds: [
      FundRecord(
        code: '012734',
        name: '易方达中证人工智能主题ETF联接C',
        groupName: 'AI / 指数增强',
        tags: '默认基金, 高频关注',
        note: '作为桌面端迁移基准对象，优先验证快照展示、规则命中与告警流。',
        snapshot: primarySnapshot,
      ),
      FundRecord(
        code: '000001',
        name: '华夏成长混合',
        groupName: '主动权益',
        tags: '观察',
        note: '当前仅展示确认净值，后续可对比“是否存在盘中估值返回”。',
        snapshot: QuoteSnapshot(
          navDate: latestNavDate,
          unitNav: '1.4820',
          confirmedChangeRate: '0.68%',
          confirmedTone: ChangeTone.up,
          estimatedNav: '-',
          estimatedChangeRate: '-',
          estimatedTone: ChangeTone.neutral,
          estimatedAt: '-',
          fetchedAt: _formatDateTime(secondaryFetchTime),
          source: '东方财富净值快照',
          hasEstimatedSnapshot: false,
        ),
      ),
      FundRecord(
        code: '006113',
        name: '广发创业板ETF联接A',
        groupName: '指数',
        tags: '轮动',
        note: '用于验证指数类基金在桌面端的多基金切换体验。',
        snapshot: QuoteSnapshot(
          navDate: latestNavDate,
          unitNav: '1.0368',
          confirmedChangeRate: '-0.45%',
          confirmedTone: ChangeTone.down,
          estimatedNav: '1.0307',
          estimatedChangeRate: '-0.59%',
          estimatedTone: ChangeTone.down,
          estimatedAt: _formatDateTime(tertiaryFetchTime),
          fetchedAt: _formatDateTime(tertiaryFetchTime),
          source: '东方财富净值快照 + 东方财富实时估值',
          hasEstimatedSnapshot: true,
        ),
      ),
    ],
    rules: const [
      RuleRecord(
        name: '涨跌幅阈值',
        scope: '012734 / 006113',
        condition: '估算涨跌幅 <= -2.50%',
        cooldown: '30 分钟',
        status: '启用',
      ),
      RuleRecord(
        name: '净值区间',
        scope: '012734',
        condition: '确认净值在 2.20 ~ 2.35',
        cooldown: '120 分钟',
        status: '启用',
      ),
      RuleRecord(
        name: '估值偏离',
        scope: '全基金池',
        condition: '估值偏离绝对值 >= 2.00%',
        cooldown: '15 分钟',
        status: '启用',
      ),
      RuleRecord(
        name: '人工观察',
        scope: '000001',
        condition: '仅保留确认净值快照',
        cooldown: '关闭',
        status: '观察中',
      ),
    ],
    alerts: [
      AlertRecord(
        title: '012734 估算跌幅已触发阈值',
        summary: '估算涨跌幅 -2.74%，已命中“涨跌幅阈值”规则。',
        level: '高',
        occurredAt: _formatDateTime(primaryFetchTime),
      ),
      AlertRecord(
        title: '000001 暂无实时估值数据',
        summary: '当前仅返回确认净值，桌面端后续会高亮这种数据缺口。',
        level: '中',
        occurredAt: _formatDateTime(secondaryFetchTime),
      ),
      AlertRecord(
        title: '006113 估值偏离接近预警线',
        summary: '估值偏离 1.84%，尚未正式触发，可继续关注。',
        level: '低',
        occurredAt: _formatDateTime(tertiaryFetchTime),
      ),
    ],
    settings: const [
      SettingGroup(
        title: '运行参数',
        items: [
          SettingItem(
            label: '桌面支持平台',
            value: 'Windows / macOS / Linux',
            hint: '当前阶段只生成桌面三端工程。',
          ),
          SettingItem(
            label: '未来移动端',
            value: 'Android / iOS',
            hint: '后续通过 flutter create --platforms=android,ios 补齐。',
          ),
          SettingItem(
            label: '模块路径',
            value: 'modules/app',
            hint: '与 fund-monitor 并列，保持 monorepo 模块结构。',
          ),
        ],
      ),
      SettingGroup(
        title: '数据与迁移策略',
        items: [
          SettingItem(
            label: '当前数据模式',
            value: '动态种子数据',
            hint: '先打通桌面刷新与时间状态，再接入真实数据源。',
          ),
          SettingItem(
            label: '目标数据来源',
            value: 'fund-monitor 领域能力',
            hint: '后续可通过共享 API / 本地桥接 / 统一存储接入。',
          ),
          SettingItem(
            label: '发布方式',
            value: 'GitHub Release',
            hint: '采用 app-v*.*.* tag 驱动的桌面构建与发布。',
          ),
        ],
      ),
    ],
    jobs: [
      JobRecord(
        name: 'poll_funds',
        status: 'success',
        startedAt: _formatDateTime(primaryFetchTime),
        note: '轮询任务已按 60 秒频率执行。',
      ),
      JobRecord(
        name: 'fund_poll_fetch:012734',
        status: 'success',
        startedAt: _formatDateTime(primaryFetchTime),
        note: '已写入最新估值快照。',
      ),
      JobRecord(
        name: 'fund_poll_fetch:000001',
        status: 'success',
        startedAt: _formatDateTime(secondaryFetchTime),
        note: '当前无盘中估值字段，仍保留确认净值。',
      ),
    ],
    lastRefreshAt: _formatDateTime(primaryFetchTime),
  );
}

DateTime _trimMilliseconds(DateTime value) {
  return DateTime(
    value.year,
    value.month,
    value.day,
    value.hour,
    value.minute,
    value.second,
  );
}

DateTime _previousTradingDay(DateTime currentTime) {
  var cursor = DateTime(
    currentTime.year,
    currentTime.month,
    currentTime.day,
  ).subtract(const Duration(days: 1));

  while (cursor.weekday == DateTime.saturday ||
      cursor.weekday == DateTime.sunday) {
    cursor = cursor.subtract(const Duration(days: 1));
  }

  return cursor;
}

String _formatDate(DateTime value) {
  final year = value.year.toString().padLeft(4, '0');
  final month = value.month.toString().padLeft(2, '0');
  final day = value.day.toString().padLeft(2, '0');
  return '$year-$month-$day';
}

String _formatDateTime(DateTime value) {
  final date = _formatDate(value);
  final hour = value.hour.toString().padLeft(2, '0');
  final minute = value.minute.toString().padLeft(2, '0');
  final second = value.second.toString().padLeft(2, '0');
  return '$date $hour:$minute:$second';
}
