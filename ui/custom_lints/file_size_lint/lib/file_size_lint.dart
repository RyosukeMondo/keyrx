import 'dart:async';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/error/error.dart';
import 'package:analyzer/error/listener.dart';
import 'package:custom_lint_builder/custom_lint_builder.dart';

/// Entry point used by custom_lint to discover plugin rules.
PluginBase createPlugin() => _FileSizeLintPlugin();

class _FileSizeLintPlugin extends PluginBase {
  @override
  List<LintRule> getLintRules(CustomLintConfigs configs) => [
    FileLineLimitLint(maxLines: 500),
    FunctionLengthLimitLint(maxLines: 50),
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
    reporter.reportErrorForOffset(code, 0, highlightLength, [
      maxLines,
      lineCount,
    ]);
  }
}

class FunctionLengthLimitLint extends DartLintRule {
  FunctionLengthLimitLint({required this.maxLines})
    : super(
        code: LintCode(
          name: 'function_length_limit',
          problemMessage: 'Function has {1} lines (max {0}).',
          correctionMessage:
              'Split the function or extract helpers to stay under {0} lines.',
        ),
      );

  final int maxLines;

  @override
  FutureOr<void> run(
    CustomLintResolver resolver,
    ErrorReporter reporter,
    CustomLintContext context,
  ) async {
    final path = resolver.path;
    if (path.endsWith('.g.dart') ||
        path.endsWith('.freezed.dart') ||
        path.contains('generated_bindings.dart')) {
      return;
    }

    final result = await resolver.getResolvedUnitResult();
    final lineInfo = result.lineInfo;

    void checkLength(AstNode node) {
      if (node is FunctionBody && node.isSynthetic) {
        return;
      }

      final start = lineInfo.getLocation(node.offset).lineNumber;
      final end = lineInfo.getLocation(node.end).lineNumber;
      final length = end - start + 1;
      if (length <= maxLines) {
        return;
      }

      reporter.reportErrorForOffset(code, node.offset, node.length, [
        maxLines,
        length,
      ]);
    }

    context.registry
      ..addFunctionDeclaration((node) {
        checkLength(node.functionExpression.body);
      })
      ..addMethodDeclaration((node) {
        checkLength(node.body);
      })
      ..addConstructorDeclaration((node) {
        if (!node.body.isSynthetic) {
          checkLength(node.body);
        }
      });
  }
}
