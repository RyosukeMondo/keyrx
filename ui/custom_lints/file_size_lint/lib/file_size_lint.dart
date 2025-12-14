import 'dart:async';
import 'package:analyzer/error/error.dart';
import 'package:analyzer/error/listener.dart';
import 'package:custom_lint_builder/custom_lint_builder.dart';

/// Entry point used by custom_lint to discover plugin rules.
PluginBase createPlugin() => _FileSizeLintPlugin();

class _FileSizeLintPlugin extends PluginBase {
  @override
  List<LintRule> getLintRules(CustomLintConfigs configs) => [
        FileLineLimitLint(maxLines: 500),
      ];
}

class FileLineLimitLint extends DartLintRule {
  FileLineLimitLint({required this.maxLines})
      : super(
          code: LintCode(
            name: 'file_line_limit',
            problemMessage: 'File has {1} lines (max {0}).',
            correctionMessage:
                'Split the file or reduce scope to stay under {0} lines.',
          ),
        );

  final int maxLines;

  @override
  FutureOr<void> run(
    CustomLintResolver resolver,
    ErrorReporter reporter,
    CustomLintContext context,
  ) async {
    // Skip generated outputs.
    final path = resolver.path;
    if (path.endsWith('.g.dart') ||
        path.endsWith('.freezed.dart') ||
        path.contains('generated_bindings.dart')) {
      return;
    }

    final result = await resolver.getResolvedUnitResult();
    final lineCount = result.lineInfo.lineCount;
    if (lineCount <= maxLines) {
      return;
    }

    // Anchor the diagnostic at the start of the file.
    final highlightLength = result.content.isNotEmpty ? 1 : 0;
    reporter.reportErrorForOffset(
      code,
      0,
      highlightLength,
      [maxLines, lineCount],
    );
  }
}
