import 'dart:io';

Future<void> main(List<String> args) async {
  if (args.length != 1) {
    stderr.writeln('Usage: dart tool/verify_release_tag.dart <release-tag>');
    exitCode = 64;
    return;
  }

  final releaseTag = args.first.trim();
  final pubspecFile = await _resolvePubspec();
  final pubspecContent = await pubspecFile.readAsString();
  final versionMatch = RegExp(
    r'^version:\s*([^\s#]+)',
    multiLine: true,
  ).firstMatch(pubspecContent);

  if (versionMatch == null) {
    stderr.writeln('Failed to read version from ${pubspecFile.path}.');
    exitCode = 1;
    return;
  }

  final packageVersion = versionMatch.group(1)!.trim();
  final releaseVersion = packageVersion.split('+').first;
  final expectedTag = 'app-v$releaseVersion';

  if (releaseTag != expectedTag) {
    stderr.writeln(
      'Release tag mismatch: expected $expectedTag, got $releaseTag.',
    );
    exitCode = 1;
    return;
  }

  stdout.writeln(
    'Verified release tag $releaseTag for app version $packageVersion.',
  );
}

Future<File> _resolvePubspec() async {
  final candidates = [File('pubspec.yaml'), File('modules/app/pubspec.yaml')];

  for (final candidate in candidates) {
    if (await candidate.exists()) {
      return candidate;
    }
  }

  throw StateError('Unable to locate modules/app/pubspec.yaml.');
}
