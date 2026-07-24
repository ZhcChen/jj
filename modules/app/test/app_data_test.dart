import 'package:flutter_test/flutter_test.dart';
import 'package:fund_monitor_app/src/app_data.dart';

void main() {
  test('desktop seed data uses current clock and previous trading day', () {
    final data = buildDesktopSeedData(now: DateTime(2026, 7, 24, 9, 31, 5));

    expect(data.lastRefreshAt, '2026-07-24 09:31:05');
    expect(data.funds.first.snapshot?.navDate, '2026-07-23');
    expect(data.alerts.first.occurredAt, '2026-07-24 09:31:05');
  });

  test('desktop seed data skips weekend when resolving nav date', () {
    final data = buildDesktopSeedData(now: DateTime(2026, 7, 27, 10, 0, 0));

    expect(data.funds.first.snapshot?.navDate, '2026-07-24');
  });
}
