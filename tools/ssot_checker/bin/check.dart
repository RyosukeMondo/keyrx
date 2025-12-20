import 'dart:io';
import 'package:args/args.dart';
import 'package:path/path.dart' as path;
import 'package:ssot_checker/ssot_checker.dart';

void main(List<String> arguments) async {
  final parser = ArgParser()
    ..addFlag(
      'verbose',
      abbr: 'v',
      negatable: false,
      help: 'Show detailed output',
    )
    ..addFlag(
      'help',
      abbr: 'h',
      negatable: false,
      help: 'Show this help message',
    )
    ..addOption(
      'path',
      abbr: 'p',
      help: 'Project root path',
      defaultsTo: '.',
    );

  ArgResults args;
  try {
    args = parser.parse(arguments);
  } catch (e) {
    print('Error: $e\n');
    _printUsage(parser);
    exit(1);
  }

  if (args['help'] as bool) {
    _printUsage(parser);
    exit(0);
  }

  final projectPath = path.normalize(path.absolute(args['path'] as String));
  final verbose = args['verbose'] as bool;

  // Verify project path exists
  if (!await Directory(projectPath).exists()) {
    print('Error: Project path does not exist: $projectPath');
    exit(1);
  }

  // Run SSOT checker
  final checker = SSOTChecker(verbose: verbose);
  final success = await checker.check(projectPath);

  // Print report
  checker.printReport();

  // Exit with appropriate code
  exit(success ? 0 : 1);
}

void _printUsage(ArgParser parser) {
  print('SSOT Violation Checker for KeyRX');
  print('');
  print('Usage: dart run ssot_checker [options]');
  print('');
  print('Options:');
  print(parser.usage);
  print('');
  print('Examples:');
  print('  dart run ssot_checker');
  print('  dart run ssot_checker --verbose');
  print('  dart run ssot_checker --path /path/to/keyrx');
}
