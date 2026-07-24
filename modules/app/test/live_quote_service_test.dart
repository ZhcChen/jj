import 'package:flutter_test/flutter_test.dart';
import 'package:fund_monitor_app/src/live_quote_service.dart';

void main() {
  test('eastmoney parser merges valuation snapshot into quote snapshot', () {
    const pingzhongdataBody =
        'var fS_name = "易方达中证人工智能主题ETF联接C";'
        'var fS_code = "012734";'
        'var Data_netWorthTrend = ['
        '{"x":1784649600000,"y":2.2499,"equityReturn":-1.26},'
        '{"x":1784736000000,"y":2.1975,"equityReturn":-2.33}'
        '];';
    const valuationBody =
        '{"data":[{"GZTIME":"2026-07-24 15:00","GSZZL":-1.78,"GSZ":2.1584}],"errorCode":0,"success":true}';

    final snapshot = parseEastmoneyQuoteSnapshot(
      fundCode: '012734',
      pingzhongdataBody: pingzhongdataBody,
      valuationBody: valuationBody,
      fetchedAt: DateTime(2026, 7, 24, 17, 21, 12),
    ).snapshot;

    expect(snapshot.navDate, '2026-07-23');
    expect(snapshot.unitNav, '2.1975');
    expect(snapshot.confirmedChangeRate, '-2.33%');
    expect(snapshot.estimatedNav, '2.1584');
    expect(snapshot.estimatedChangeRate, '-1.78%');
    expect(snapshot.estimatedAt, '2026-07-24 15:00:00');
    expect(snapshot.fetchedAt, '2026-07-24 17:21:12');
    expect(snapshot.source, '东方财富净值快照 + 东方财富实时估值');
    expect(snapshot.hasEstimatedSnapshot, isTrue);
  });

  test('eastmoney parser rejects mismatched fund code', () {
    const pingzhongdataBody =
        'var fS_name = "易方达中证人工智能主题ETF联接C";'
        'var fS_code = "999999";'
        'var Data_netWorthTrend = [{"x":1784736000000,"y":2.1975,"equityReturn":-2.33}];';

    expect(
      () => parseEastmoneyQuoteSnapshot(
        fundCode: '012734',
        pingzhongdataBody: pingzhongdataBody,
      ),
      throwsA(isA<FormatException>()),
    );
  });
}
