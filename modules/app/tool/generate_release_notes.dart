import 'dart:io';

Future<void> main(List<String> args) async {
  if (args.length != 4) {
    stderr.writeln(
      'Usage: dart tool/generate_release_notes.dart <release-tag> <github-repository> <asset-dir> <output-file>',
    );
    exitCode = 64;
    return;
  }

  final releaseTag = args[0].trim();
  final githubRepository = args[1].trim();
  final assetDir = Directory(args[2]);
  final outputFile = File(args[3]);

  if (!await assetDir.exists()) {
    stderr.writeln('Asset directory not found: ${assetDir.path}');
    exitCode = 1;
    return;
  }

  final assetNames = await assetDir
      .list()
      .where((entity) => entity is File)
      .cast<File>()
      .map((file) => file.uri.pathSegments.last)
      .toList();
  assetNames.sort();

  if (assetNames.isEmpty) {
    stderr.writeln('No release assets found under ${assetDir.path}.');
    exitCode = 1;
    return;
  }

  final releaseBaseUrl =
      'https://github.com/$githubRepository/releases/download/$releaseTag';
  final groupedAssets = _groupAssets(assetNames, releaseBaseUrl);
  final previousTag = await _findPreviousTag(releaseTag);

  final buffer = StringBuffer()
    ..writeln('# Fund Monitor App $releaseTag')
    ..writeln()
    ..writeln('## 下载')
    ..writeln();

  for (final platform in _platformOrder) {
    final platformAssets = groupedAssets[platform];
    if (platformAssets == null || platformAssets.isEmpty) {
      continue;
    }

    buffer.writeln('### $platform');

    for (final arch in _archOrder) {
      final archAssets = platformAssets[arch];
      if (archAssets == null || archAssets.isEmpty) {
        continue;
      }

      buffer.writeln('- $arch');

      for (final asset in archAssets) {
        final formatLabel = _formatLabels[asset.format] ?? asset.format;
        buffer.writeln('  - [$formatLabel](${asset.url}) - `${asset.name}`');
      }
    }

    buffer.writeln();
  }

  buffer
    ..writeln('## 当前范围')
    ..writeln()
    ..writeln('- Flutter 桌面端基础工作区')
    ..writeln('- 当前先支持 Windows / macOS / Linux')
    ..writeln('- 当前桌面数据为迁移期种子数据，后续继续接入真实基金监控链路');

  if (previousTag != null) {
    buffer
      ..writeln()
      ..writeln('## 变更')
      ..writeln()
      ..writeln(
        '- [查看完整变更](https://github.com/$githubRepository/compare/$previousTag...$releaseTag)',
      );
  }

  await outputFile.parent.create(recursive: true);
  await outputFile.writeAsString(buffer.toString());
  stdout.writeln('Release notes written to ${outputFile.path}');
}

const _platformOrder = ['macOS', 'Windows', 'Linux'];
const _archOrder = ['arm64', 'x64'];
const _formatOrder = ['dmg', 'exe', 'deb', 'appimage', 'zip'];
const _formatLabels = {
  'appimage': 'AppImage',
  'deb': 'DEB',
  'dmg': 'DMG',
  'exe': 'EXE',
  'zip': 'ZIP',
};

Map<String, Map<String, List<_ReleaseAsset>>> _groupAssets(
  List<String> assetNames,
  String releaseBaseUrl,
) {
  final grouped = <String, Map<String, List<_ReleaseAsset>>>{};

  for (final name in assetNames) {
    final normalized = name.toLowerCase();
    final platform = _detectPlatform(normalized);
    final arch = _detectArch(normalized);
    final format = _detectFormat(normalized);

    if (platform == null || arch == null || format == null) {
      continue;
    }

    final platformGroup = grouped.putIfAbsent(platform, () => {});
    final archGroup = platformGroup.putIfAbsent(arch, () => []);
    archGroup.add(
      _ReleaseAsset(
        name: name,
        format: format,
        url: '$releaseBaseUrl/${Uri.encodeComponent(name)}',
      ),
    );
  }

  for (final platformGroup in grouped.values) {
    for (final archGroup in platformGroup.values) {
      archGroup.sort((left, right) {
        final leftOrder = _formatOrder.indexOf(left.format);
        final rightOrder = _formatOrder.indexOf(right.format);
        return leftOrder.compareTo(rightOrder);
      });
    }
  }

  return grouped;
}

String? _detectPlatform(String fileName) {
  if (fileName.contains('-macos-')) {
    return 'macOS';
  }

  if (fileName.contains('-windows-')) {
    return 'Windows';
  }

  if (fileName.contains('-linux-')) {
    return 'Linux';
  }

  return null;
}

String? _detectArch(String fileName) {
  if (fileName.contains('-arm64.')) {
    return 'arm64';
  }

  if (fileName.contains('-x64.') ||
      fileName.contains('-amd64.') ||
      fileName.contains('-x86_64.')) {
    return 'x64';
  }

  if (fileName.contains('-aarch64.')) {
    return 'arm64';
  }

  return null;
}

String? _detectFormat(String fileName) {
  if (fileName.endsWith('.appimage')) {
    return 'appimage';
  }

  if (fileName.endsWith('.deb')) {
    return 'deb';
  }

  if (fileName.endsWith('.dmg')) {
    return 'dmg';
  }

  if (fileName.endsWith('.exe')) {
    return 'exe';
  }

  if (fileName.endsWith('.zip')) {
    return 'zip';
  }

  return null;
}

Future<String?> _findPreviousTag(String currentTag) async {
  final result = await Process.run('git', [
    'tag',
    '--list',
    'app-v*.*.*',
    '--sort=-version:refname',
  ]);

  if (result.exitCode != 0) {
    return null;
  }

  final tags = result.stdout
      .toString()
      .split('\n')
      .map((tag) => tag.trim())
      .where((tag) => tag.isNotEmpty)
      .toList();

  final currentIndex = tags.indexOf(currentTag);
  if (currentIndex == -1) {
    return null;
  }

  return currentIndex + 1 < tags.length ? tags[currentIndex + 1] : null;
}

class _ReleaseAsset {
  const _ReleaseAsset({
    required this.name,
    required this.format,
    required this.url,
  });

  final String name;
  final String format;
  final String url;
}
