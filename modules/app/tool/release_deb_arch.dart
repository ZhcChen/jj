import 'dart:io';

Future<void> main(List<String> args) async {
  if (args.length != 1) {
    stderr.writeln('Usage: dart tool/release_deb_arch.dart <arch>');
    exitCode = 64;
    return;
  }

  final arch = _normalizeDebArch(args[0].trim());
  if (arch == null) {
    stderr.writeln('Unsupported Debian arch: ${args[0]}.');
    exitCode = 1;
    return;
  }

  stdout.writeln(arch);
}

String? _normalizeDebArch(String raw) {
  switch (raw.toLowerCase()) {
    case 'x64':
    case 'x86_64':
    case 'amd64':
      return 'amd64';
    case 'arm64':
    case 'aarch64':
      return 'arm64';
    default:
      return null;
  }
}
