import 'package:flutter_test/flutter_test.dart';
import 'package:fund_monitor_app/src/app.dart';

void main() {
  testWidgets('desktop shell shows sections and read-only fund detail', (
    WidgetTester tester,
  ) async {
    await tester.pumpWidget(
      const FundMonitorDesktopApp(enableLiveQuotes: false),
    );

    expect(find.text('基金监控桌面端'), findsOneWidget);
    expect(find.text('总览看板'), findsAtLeastNWidgets(1));

    await tester.tap(find.text('基金列表').first);
    await tester.pumpAndSettle();

    expect(find.text('基金详情'), findsAtLeastNWidgets(1));
    expect(find.text('只读'), findsAtLeastNWidgets(1));
    expect(find.text('易方达中证人工智能主题ETF联接C'), findsAtLeastNWidgets(1));
  });
}
