import 'dart:io';

Future<void> main(List<String> args) async {
  if (args.length != 3) {
    stderr.writeln(
      'Usage: dart tool/generate_release_notes.dart <release-tag> <asset-dir> <output-file>',
    );
    exitCode = 64;
    return;
  }

  final releaseTag = args[0].trim();
  final assetDir = Directory(args[1]);
  final outputFile = File(args[2]);

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

  final buffer = StringBuffer()
    ..writeln('# Fund Monitor App $releaseTag')
    ..writeln()
    ..writeln('## 下载内容')
    ..writeln();

  for (final name in assetNames) {
    buffer.writeln('- `$name`');
  }

  buffer
    ..writeln()
    ..writeln('## 当前范围')
    ..writeln()
    ..writeln('- Flutter 桌面端基础工作区')
    ..writeln('- 当前先支持 Windows / macOS / Linux')
    ..writeln('- 当前桌面数据为迁移期种子数据，后续继续接入真实基金监控链路');

  await outputFile.parent.create(recursive: true);
  await outputFile.writeAsString(buffer.toString());
  stdout.writeln('Release notes written to ${outputFile.path}');
}
