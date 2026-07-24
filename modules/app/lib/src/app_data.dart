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

  FundRecord copyWith({QuoteSnapshot? snapshot}) {
    return FundRecord(
      code: code,
      name: name,
      groupName: groupName,
      tags: tags,
      note: note,
      snapshot: snapshot ?? this.snapshot,
    );
  }
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

DesktopSeedData buildDesktopSeedData({
  DateTime? now,
  List<FundRecord>? funds,
  DateTime? refreshedAt,
  String? dataMode,
}) {
  final effectiveFunds = funds ?? buildDesktopFundCatalog();
  final effectiveAlerts = _buildAlertRecords(effectiveFunds, refreshedAt);
  final actionableAlertCount = _countActionableAlerts(effectiveFunds);

  return DesktopSeedData(
    metrics: [
      MetricCardData(
        label: '启用基金',
        value: '${effectiveFunds.length}',
        hint: '当前桌面端展示的监控基金池规模',
      ),
      const MetricCardData(
        label: '轮询频率',
        value: '60 秒',
        hint: '桌面端与监控任务统一按分钟刷新',
      ),
      const MetricCardData(label: '活跃规则', value: '4', hint: '已启用的监控规则数量'),
      MetricCardData(
        label: '待处理告警',
        value: '$actionableAlertCount',
        hint: refreshedAt == null ? '等待首轮实时抓取完成后生成' : '基于最近一次实时快照自动汇总',
      ),
    ],
    funds: effectiveFunds,
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
    alerts: effectiveAlerts,
    settings: [
      const SettingGroup(
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
            value: dataMode ?? '等待首轮实时抓取',
            hint: '当前由桌面端直接请求东方财富净值与估值接口。',
          ),
          const SettingItem(
            label: '目标数据来源',
            value: '东方财富净值快照 / 实时估值',
            hint: '当前已直连实时行情，后续再评估是否复用 fund-monitor 本地存储。',
          ),
          const SettingItem(
            label: '发布方式',
            value: 'GitHub Release',
            hint: '采用 app-v*.*.* tag 驱动的桌面构建与发布。',
          ),
        ],
      ),
    ],
    jobs: _buildJobRecords(effectiveFunds, refreshedAt),
    lastRefreshAt: refreshedAt == null
        ? '等待首轮抓取'
        : formatDesktopDateTime(refreshedAt),
  );
}

List<FundRecord> buildDesktopFundCatalog() {
  return const [
    FundRecord(
      code: '012734',
      name: '易方达中证人工智能主题ETF联接C',
      groupName: 'AI / 指数增强',
      tags: '默认基金, 高频关注',
      note: '默认对齐基金，优先校验实时估值、净值日期与分钟级刷新。',
      snapshot: null,
    ),
    FundRecord(
      code: '000001',
      name: '华夏成长混合',
      groupName: '主动权益',
      tags: '观察',
      note: '用于验证主动权益基金在仅有确认净值时的只读展示。',
      snapshot: null,
    ),
    FundRecord(
      code: '006113',
      name: '广发创业板ETF联接A',
      groupName: '指数',
      tags: '轮动',
      note: '用于验证指数类基金在桌面端的多基金切换与实时估值刷新。',
      snapshot: null,
    ),
  ];
}

List<AlertRecord> _buildAlertRecords(
  List<FundRecord> funds,
  DateTime? refreshedAt,
) {
  if (refreshedAt == null) {
    return const [
      AlertRecord(
        title: '等待首轮实时行情',
        summary: '应用启动后会自动抓取基金净值与盘中估值快照。',
        level: '中',
        occurredAt: '-',
      ),
    ];
  }

  final alerts = <AlertRecord>[];

  for (final fund in funds) {
    final snapshot = fund.snapshot;
    if (snapshot == null) {
      alerts.add(
        AlertRecord(
          title: '${fund.code} 暂无最新快照',
          summary: '最近一次刷新未拿到 ${fund.name} 的实时行情，请继续观察下一轮抓取。',
          level: '高',
          occurredAt: formatDesktopDateTime(refreshedAt),
        ),
      );
      continue;
    }

    if (!snapshot.hasEstimatedSnapshot) {
      alerts.add(
        AlertRecord(
          title: '${fund.code} 暂无盘中估值',
          summary: '${fund.name} 当前仅返回确认净值，暂未获得实时估值字段。',
          level: '中',
          occurredAt: snapshot.fetchedAt,
        ),
      );
      continue;
    }

    final estimatedChangeRate = _parsePercent(snapshot.estimatedChangeRate);
    if (estimatedChangeRate != null && estimatedChangeRate.abs() >= 2.0) {
      alerts.add(
        AlertRecord(
          title: '${fund.code} 盘中波动达到关注阈值',
          summary:
              '${fund.name} 当前估算涨跌幅 ${snapshot.estimatedChangeRate}，建议结合持仓继续跟踪。',
          level: estimatedChangeRate.abs() >= 2.5 ? '高' : '中',
          occurredAt: snapshot.estimatedAt == '-'
              ? snapshot.fetchedAt
              : snapshot.estimatedAt,
        ),
      );
    }
  }

  if (alerts.isEmpty) {
    return [
      AlertRecord(
        title: '暂无异常告警',
        summary: '最近一次实时抓取未发现需要立即处理的行情异常。',
        level: '低',
        occurredAt: formatDesktopDateTime(refreshedAt),
      ),
    ];
  }

  return alerts.take(3).toList();
}

int _countActionableAlerts(List<FundRecord> funds) {
  var count = 0;

  for (final fund in funds) {
    final snapshot = fund.snapshot;
    if (snapshot == null || !snapshot.hasEstimatedSnapshot) {
      count += 1;
      continue;
    }

    final estimatedChangeRate = _parsePercent(snapshot.estimatedChangeRate);
    if (estimatedChangeRate != null && estimatedChangeRate.abs() >= 2.0) {
      count += 1;
    }
  }

  return count;
}

List<JobRecord> _buildJobRecords(
  List<FundRecord> funds,
  DateTime? refreshedAt,
) {
  if (refreshedAt == null) {
    return const [
      JobRecord(
        name: 'poll_funds',
        status: 'pending',
        startedAt: '-',
        note: '等待首轮实时行情抓取。',
      ),
    ];
  }

  final successCount = funds.where((fund) => fund.snapshot != null).length;
  final jobs = <JobRecord>[
    JobRecord(
      name: 'poll_funds',
      status: successCount == funds.length ? 'success' : 'partial',
      startedAt: formatDesktopDateTime(refreshedAt),
      note: '已完成 $successCount/${funds.length} 只基金的实时行情刷新。',
    ),
  ];

  for (final fund in funds.take(2)) {
    final snapshot = fund.snapshot;
    jobs.add(
      JobRecord(
        name: 'fund_poll_fetch:${fund.code}',
        status: snapshot == null ? 'failed' : 'success',
        startedAt: snapshot?.fetchedAt ?? formatDesktopDateTime(refreshedAt),
        note: snapshot == null
            ? '本轮未拿到实时数据，继续保留上一轮可见状态。'
            : snapshot.hasEstimatedSnapshot
            ? '已更新确认净值与盘中估值。'
            : '已更新确认净值，盘中估值暂不可用。',
      ),
    );
  }

  return jobs;
}

double? _parsePercent(String value) {
  if (value == '-') {
    return null;
  }

  return double.tryParse(value.replaceAll('%', '').trim());
}

String formatDesktopDate(DateTime value) {
  final year = value.year.toString().padLeft(4, '0');
  final month = value.month.toString().padLeft(2, '0');
  final day = value.day.toString().padLeft(2, '0');
  return '$year-$month-$day';
}

String formatDesktopDateTime(DateTime value) {
  final date = formatDesktopDate(value);
  final hour = value.hour.toString().padLeft(2, '0');
  final minute = value.minute.toString().padLeft(2, '0');
  final second = value.second.toString().padLeft(2, '0');
  return '$date $hour:$minute:$second';
}
