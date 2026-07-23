import 'dart:io';

Future<void> main(List<String> args) async {
  if (args.length != 4) {
    stderr.writeln(
      'Usage: dart tool/release_asset_name.dart <version> <platform> <arch> <extension>',
    );
    exitCode = 64;
    return;
  }

  final version = args[0].trim();
  final platform = _normalizePlatform(args[1].trim());
  final arch = _normalizeArch(args[2].trim());
  final extension = args[3].trim().toLowerCase();

  if (version.isEmpty) {
    stderr.writeln('Version must not be empty.');
    exitCode = 1;
    return;
  }

  if (platform == null) {
    stderr.writeln('Unsupported platform: ${args[1]}.');
    exitCode = 1;
    return;
  }

  if (arch == null) {
    stderr.writeln('Unsupported arch: ${args[2]}.');
    exitCode = 1;
    return;
  }

  if (extension.isEmpty) {
    stderr.writeln('Extension must not be empty.');
    exitCode = 1;
    return;
  }

  stdout.writeln('fund-monitor-app-$version-$platform-$arch.$extension');
}

String? _normalizePlatform(String raw) {
  switch (raw.toLowerCase()) {
    case 'linux':
      return 'linux';
    case 'mac':
    case 'macos':
      return 'macos';
    case 'win':
    case 'windows':
      return 'windows';
    default:
      return null;
  }
}

String? _normalizeArch(String raw) {
  switch (raw.toLowerCase()) {
    case 'x64':
    case 'x86_64':
    case 'amd64':
      return 'x64';
    case 'arm64':
    case 'aarch64':
      return 'arm64';
    default:
      return null;
  }
}
