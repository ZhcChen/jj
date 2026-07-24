import 'dart:convert';
import 'dart:io';

import 'app_data.dart';

const String _eastmoneyBaseUrl = 'https://fund.eastmoney.com';
const String _valuationPrimaryBaseUrl = 'https://fundcomapi.eastmoney.com';
const String _valuationFallbackBaseUrl = 'https://fundcomapi.tiantianfunds.com';
const String _valuationPath =
    'mm/newCore/FundValuationLast'
    '?deviceid=1234567.py.service'
    '&version=6.5.5'
    '&appVersion=6.5.5'
    '&product=EFund'
    '&plat=Iphone';

final RegExp _namePattern = RegExp(r'var\s+fS_name\s*=\s*"([^"]+)";');
final RegExp _codePattern = RegExp(r'var\s+fS_code\s*=\s*"([^"]+)";');
final RegExp _netWorthPattern = RegExp(
  r'var\s+Data_netWorthTrend\s*=\s*(\[[^;]*\]);',
);
final RegExp _estimatedNavPattern = RegExp(r'var\s+gsz\s*=\s*"([^"]+)";');
final RegExp _estimatedChangeRatePattern = RegExp(
  r'var\s+gszzl\s*=\s*"([^"]+)";',
);
final RegExp _estimatedAtPattern = RegExp(r'var\s+gztime\s*=\s*"([^"]+)";');

class FundRefreshResult {
  const FundRefreshResult({required this.funds, required this.refreshedAt});

  final List<FundRecord> funds;
  final DateTime? refreshedAt;
}

class LiveQuoteSnapshot {
  const LiveQuoteSnapshot({required this.snapshot, required this.fetchedAt});

  final QuoteSnapshot snapshot;
  final DateTime fetchedAt;
}

class EastmoneyQuoteService {
  EastmoneyQuoteService({HttpClient? httpClient})
    : _httpClient = httpClient ?? HttpClient() {
    _httpClient.userAgent = 'fund-monitor-app/0.1.1';
    _httpClient.connectionTimeout = const Duration(seconds: 10);
  }

  final HttpClient _httpClient;

  Future<FundRefreshResult> refreshFunds(List<FundRecord> funds) async {
    final outcomes = await Future.wait(funds.map(_refreshFund));

    DateTime? refreshedAt;
    for (final outcome in outcomes) {
      if (outcome.fetchedAt == null) {
        continue;
      }

      if (refreshedAt == null || outcome.fetchedAt!.isAfter(refreshedAt)) {
        refreshedAt = outcome.fetchedAt;
      }
    }

    return FundRefreshResult(
      funds: outcomes.map((outcome) => outcome.fund).toList(),
      refreshedAt: refreshedAt,
    );
  }

  Future<LiveQuoteSnapshot> fetchQuoteSnapshot(String fundCode) async {
    final pingzhongdataUri = Uri.parse(
      '$_eastmoneyBaseUrl/pingzhongdata/$fundCode.js?v=${DateTime.now().millisecondsSinceEpoch}',
    );
    final pingzhongdataBody = await _getText(pingzhongdataUri);
    final valuationBody = await _fetchValuationBody(fundCode);

    return parseEastmoneyQuoteSnapshot(
      fundCode: fundCode,
      pingzhongdataBody: pingzhongdataBody,
      valuationBody: valuationBody,
      fetchedAt: DateTime.now(),
    );
  }

  void close() {
    _httpClient.close(force: true);
  }

  Future<_FundRefreshOutcome> _refreshFund(FundRecord fund) async {
    try {
      final liveSnapshot = await fetchQuoteSnapshot(fund.code);
      return _FundRefreshOutcome(
        fund: fund.copyWith(snapshot: liveSnapshot.snapshot),
        fetchedAt: liveSnapshot.fetchedAt,
      );
    } on Object {
      return _FundRefreshOutcome(fund: fund, fetchedAt: null);
    }
  }

  Future<String?> _fetchValuationBody(String fundCode) async {
    final body = 'FCODES=$fundCode&FIELDS=GSZZL,GZTIME,GSZ';

    for (final baseUrl in [
      _valuationPrimaryBaseUrl,
      _valuationFallbackBaseUrl,
    ]) {
      final uri = Uri.parse('$baseUrl/$_valuationPath');

      try {
        return await _postFormText(uri, body);
      } on Object {
        continue;
      }
    }

    return null;
  }

  Future<String> _getText(Uri uri) async {
    final request = await _httpClient.getUrl(uri);
    final response = await request.close();
    return _readResponseBody(response);
  }

  Future<String> _postFormText(Uri uri, String body) async {
    final request = await _httpClient.postUrl(uri);
    request.headers.set(
      HttpHeaders.contentTypeHeader,
      'application/x-www-form-urlencoded; charset=UTF-8',
    );
    request.write(body);
    final response = await request.close();
    return _readResponseBody(response);
  }

  Future<String> _readResponseBody(HttpClientResponse response) async {
    if (response.statusCode < 200 || response.statusCode >= 300) {
      throw HttpException(
        'Unexpected status code: ${response.statusCode}',
        uri: response.redirects.isNotEmpty
            ? response.redirects.last.location
            : null,
      );
    }

    final bytes = <int>[];
    await for (final chunk in response) {
      bytes.addAll(chunk);
    }
    return utf8.decode(bytes, allowMalformed: true);
  }
}

LiveQuoteSnapshot parseEastmoneyQuoteSnapshot({
  required String fundCode,
  required String pingzhongdataBody,
  String? valuationBody,
  DateTime? fetchedAt,
}) {
  final effectiveFetchedAt = fetchedAt ?? DateTime.now();
  final normalized = pingzhongdataBody.startsWith('\u{feff}')
      ? pingzhongdataBody.substring(1)
      : pingzhongdataBody;
  final code = _captureRequired(_codePattern, normalized, '基金代码');
  if (code != fundCode) {
    throw FormatException('基金代码不匹配：期望 $fundCode，实际 $code');
  }

  _captureRequired(_namePattern, normalized, '基金名称');

  final historyJson = _captureRequired(_netWorthPattern, normalized, '净值历史');
  final history = (jsonDecode(historyJson) as List<dynamic>)
      .cast<Map<String, dynamic>>();
  if (history.isEmpty) {
    throw const FormatException('基金数据源未返回任何净值历史');
  }

  final latest = history.last;
  final latestTimestampMs = _asInt(latest['x']);
  final latestUnitNav = _asDouble(latest['y']);
  if (latestTimestampMs == null || latestUnitNav == null) {
    throw const FormatException('基金净值历史缺少最新净值数据');
  }

  final navDate = formatDesktopDate(
    _eastmoneyTradingDateFromTimestamp(latestTimestampMs),
  );
  final confirmedChangeRate =
      _asDouble(latest['equityReturn']) ?? _computeNavChangeRate(history);
  var estimatedNav = _captureOptionalDecimal(_estimatedNavPattern, normalized);
  var estimatedChangeRate = _captureOptionalDecimal(
    _estimatedChangeRatePattern,
    normalized,
  );
  var estimatedAt = _captureOptionalLocalDateTime(
    _estimatedAtPattern,
    normalized,
  );
  var sourceLabel = '东方财富净值快照';

  final valuationSnapshot = _parseEastmoneyValuationSnapshot(valuationBody);
  if (valuationSnapshot != null) {
    estimatedNav = valuationSnapshot.estimatedNav ?? estimatedNav;
    estimatedChangeRate =
        valuationSnapshot.estimatedChangeRate ??
        _computeEstimatedChangeRate(
          latestUnitNav,
          valuationSnapshot.estimatedNav,
        ) ??
        estimatedChangeRate;
    estimatedAt = valuationSnapshot.estimatedAt ?? estimatedAt;
    sourceLabel = '东方财富净值快照 + 东方财富实时估值';
  }

  final hasEstimatedSnapshot =
      estimatedNav != null ||
      estimatedChangeRate != null ||
      estimatedAt != null;

  return LiveQuoteSnapshot(
    fetchedAt: effectiveFetchedAt,
    snapshot: QuoteSnapshot(
      navDate: navDate,
      unitNav: latestUnitNav.toStringAsFixed(4),
      confirmedChangeRate: _formatPercent(confirmedChangeRate),
      confirmedTone: _toneForRate(confirmedChangeRate),
      estimatedNav: _formatDecimal(estimatedNav),
      estimatedChangeRate: _formatPercent(estimatedChangeRate),
      estimatedTone: _toneForRate(estimatedChangeRate),
      estimatedAt: estimatedAt == null
          ? '-'
          : formatDesktopDateTime(estimatedAt),
      fetchedAt: formatDesktopDateTime(effectiveFetchedAt),
      source: sourceLabel,
      hasEstimatedSnapshot: hasEstimatedSnapshot,
    ),
  );
}

_ValuationSnapshot? _parseEastmoneyValuationSnapshot(String? body) {
  if (body == null || body.trim().isEmpty) {
    return null;
  }

  final decoded = jsonDecode(body) as Map<String, dynamic>;
  final errorCode = _asInt(decoded['errorCode']) ?? 0;
  if (errorCode != 0) {
    return null;
  }

  final data = decoded['data'];
  if (data is! List || data.isEmpty) {
    return null;
  }

  final item = data.first;
  if (item is! Map<String, dynamic>) {
    return null;
  }

  final estimatedNav = _asDouble(item['GSZ']);
  final estimatedChangeRate = _asDouble(item['GSZZL']);
  final estimatedAtValue = item['GZTIME'];
  final estimatedAt = estimatedAtValue is String
      ? _parseEastmoneyLocalDateTime(estimatedAtValue)
      : null;

  if (estimatedNav == null && estimatedChangeRate == null) {
    return null;
  }

  return _ValuationSnapshot(
    estimatedNav: estimatedNav,
    estimatedChangeRate: estimatedChangeRate,
    estimatedAt: estimatedAt,
  );
}

class _FundRefreshOutcome {
  const _FundRefreshOutcome({required this.fund, required this.fetchedAt});

  final FundRecord fund;
  final DateTime? fetchedAt;
}

class _ValuationSnapshot {
  const _ValuationSnapshot({
    required this.estimatedNav,
    required this.estimatedChangeRate,
    required this.estimatedAt,
  });

  final double? estimatedNav;
  final double? estimatedChangeRate;
  final DateTime? estimatedAt;
}

String _captureRequired(RegExp pattern, String body, String fieldName) {
  final value = pattern.firstMatch(body)?.group(1)?.trim();
  if (value == null || value.isEmpty) {
    throw FormatException('基金数据源缺少必要字段：$fieldName');
  }

  return value;
}

double? _captureOptionalDecimal(RegExp pattern, String body) {
  final value = pattern.firstMatch(body)?.group(1)?.trim();
  if (value == null || value.isEmpty) {
    return null;
  }

  return double.tryParse(value);
}

DateTime? _captureOptionalLocalDateTime(RegExp pattern, String body) {
  final value = pattern.firstMatch(body)?.group(1)?.trim();
  if (value == null || value.isEmpty) {
    return null;
  }

  return _parseEastmoneyLocalDateTime(value);
}

DateTime _eastmoneyTradingDateFromTimestamp(int timestampMs) {
  return DateTime.fromMillisecondsSinceEpoch(
    timestampMs,
    isUtc: true,
  ).add(const Duration(hours: 8));
}

DateTime? _parseEastmoneyLocalDateTime(String value) {
  final normalized = value.trim();
  final segments = normalized.split(' ');
  if (segments.length != 2) {
    return null;
  }

  final dateParts = segments[0].split('-');
  final timeParts = segments[1].split(':');
  if (dateParts.length != 3 || timeParts.length < 2 || timeParts.length > 3) {
    return null;
  }

  final year = int.tryParse(dateParts[0]);
  final month = int.tryParse(dateParts[1]);
  final day = int.tryParse(dateParts[2]);
  final hour = int.tryParse(timeParts[0]);
  final minute = int.tryParse(timeParts[1]);
  final second = timeParts.length == 3 ? int.tryParse(timeParts[2]) ?? 0 : 0;

  if (year == null ||
      month == null ||
      day == null ||
      hour == null ||
      minute == null) {
    return null;
  }

  return DateTime(year, month, day, hour, minute, second);
}

double? _computeEstimatedChangeRate(double? unitNav, double? estimatedNav) {
  if (unitNav == null || estimatedNav == null || unitNav == 0) {
    return null;
  }

  return ((estimatedNav - unitNav) / unitNav) * 100;
}

double? _computeNavChangeRate(List<Map<String, dynamic>> history) {
  if (history.length < 2) {
    return null;
  }

  final previous = _asDouble(history[history.length - 2]['y']);
  final latest = _asDouble(history.last['y']);
  if (previous == null || latest == null || previous == 0) {
    return null;
  }

  return ((latest - previous) / previous) * 100;
}

ChangeTone _toneForRate(double? rate) {
  if (rate == null) {
    return ChangeTone.neutral;
  }
  if (rate > 0) {
    return ChangeTone.up;
  }
  if (rate < 0) {
    return ChangeTone.down;
  }
  return ChangeTone.neutral;
}

String _formatDecimal(double? value) {
  if (value == null) {
    return '-';
  }
  return value.toStringAsFixed(4);
}

String _formatPercent(double? value) {
  if (value == null) {
    return '-';
  }
  return '${value.toStringAsFixed(2)}%';
}

double? _asDouble(Object? value) {
  if (value is num) {
    return value.toDouble();
  }
  if (value is String) {
    return double.tryParse(value);
  }
  return null;
}

int? _asInt(Object? value) {
  if (value is int) {
    return value;
  }
  if (value is num) {
    return value.toInt();
  }
  if (value is String) {
    return int.tryParse(value);
  }
  return null;
}
