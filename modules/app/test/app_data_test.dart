import 'package:flutter_test/flutter_test.dart';
import 'package:fund_monitor_app/src/app_data.dart';

void main() {
  test('desktop data waits for first live refresh by default', () {
    final data = buildDesktopSeedData(funds: buildDesktopFundCatalog());

    expect(data.lastRefreshAt, '等待首轮抓取');
    expect(data.funds.first.snapshot, isNull);
    expect(data.alerts.first.title, '等待首轮实时行情');
    expect(data.jobs.first.status, 'pending');
  });

  test('desktop data reflects live refresh metadata after quote update', () {
    final funds = buildDesktopFundCatalog();
    final liveFunds = [
      funds.first.copyWith(
        snapshot: const QuoteSnapshot(
          navDate: '2026-07-23',
          unitNav: '2.1975',
          confirmedChangeRate: '-2.33%',
          confirmedTone: ChangeTone.down,
          estimatedNav: '2.1584',
          estimatedChangeRate: '-1.78%',
          estimatedTone: ChangeTone.down,
          estimatedAt: '2026-07-24 15:00:00',
          fetchedAt: '2026-07-24 17:21:12',
          source: '东方财富净值快照 + 东方财富实时估值',
          hasEstimatedSnapshot: true,
        ),
      ),
      funds[1],
      funds[2],
    ];

    final data = buildDesktopSeedData(
      funds: liveFunds,
      refreshedAt: DateTime(2026, 7, 24, 17, 21, 12),
      dataMode: '实时行情直连（东方财富）',
    );

    expect(data.lastRefreshAt, '2026-07-24 17:21:12');
    expect(data.settings.last.items.first.value, '实时行情直连（东方财富）');
    expect(data.metrics.last.value, '2');
  });
}
