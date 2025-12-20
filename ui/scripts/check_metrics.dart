import 'dart:io';

import 'package:analyzer/dart/analysis/utilities.dart';
import 'package:analyzer/dart/ast/ast.dart';
import 'package:analyzer/dart/ast/visitor.dart';
import 'package:analyzer/source/line_info.dart';
import 'package:path/path.dart' as p;

const _maxFileLines = 500;
const _maxFunctionLines = 50;
const _ignoreFileListPath = 'scripts/metrics_ignore.txt';

/// Checks Dart files in `lib/` for file and function length limits.
/// Usage: `cd ui && dart run scripts/check_metrics.dart`
Future<void> main(List<String> args) async {
  final libDir = Directory('lib');
  if (!libDir.existsSync()) {
    stderr.writeln('Error: lib/ directory not found. Run from the ui/ workspace root.');
    exitCode = 1;
    return;
  }

  final ignorePaths = _loadIgnoreList();
  final files = _collectDartFiles(libDir, ignorePaths);
  final violations = <String>[];

  for (final file in files) {
    final relativePath = p.relative(file.path, from: Directory.current.path);
    final normalizedPath = _normalizePath(relativePath);
    if (ignorePaths.contains(normalizedPath)) {
      continue;
    }

    final content = await file.readAsString();
    final parsed = parseString(path: file.path, throwIfDiagnostics: false, content: content);
    final lineInfo = parsed.lineInfo;

    final fileLines = lineInfo.lineCount;
    if (fileLines > _maxFileLines) {
      violations.add('$relativePath exceeds $_maxFileLines lines (found $fileLines)');
    }

    final visitor = _FunctionLengthVisitor(
      lineInfo: lineInfo,
      maxLines: _maxFunctionLines,
      onViolation: (message) => violations.add('$relativePath: $message'),
    );
    parsed.unit.visitChildren(visitor);
  }

  if (violations.isEmpty) {
    stdout.writeln(
      '✅ Metrics check passed for ${files.length} files '
      '(max file $_maxFileLines lines, function $_maxFunctionLines lines).',
    );
    return;
  }

  stdout.writeln('❌ Metrics check failed (${violations.length} issue(s)):\n');
  for (final violation in violations) {
    stdout.writeln(' - $violation');
  }
  exitCode = 1;
}

List<File> _collectDartFiles(Directory libDir, Set<String> ignorePaths) {
  final files =
      libDir
          .listSync(recursive: true)
          .whereType<File>()
          .where((file) => file.path.endsWith('.dart'))
          .where((file) => !_isGenerated(file.path))
          .where(
            (file) => !ignorePaths.contains(
              _normalizePath(p.relative(file.path, from: Directory.current.path)),
            ),
          )
          .toList()
        ..sort((a, b) => a.path.compareTo(b.path));
  return files;
}

bool _isGenerated(String path) =>
    path.endsWith('.g.dart') ||
    path.endsWith('.freezed.dart') ||
    path.contains('generated_bindings.dart') ||
    path.contains('bindings_generated.dart');

Set<String> _loadIgnoreList() {
  final file = File(_ignoreFileListPath);
  if (!file.existsSync()) {
    return {};
  }

  final lines = file
      .readAsLinesSync()
      .map((line) => line.trim())
      .where((line) => line.isNotEmpty && !line.startsWith('#'))
      .map(_normalizePath)
      .toSet();

  return lines;
}

String _normalizePath(String path) => path.replaceAll(r'\', '/');

class _FunctionLengthVisitor extends RecursiveAstVisitor<void> {
  _FunctionLengthVisitor({
    required this.lineInfo,
    required this.maxLines,
    required this.onViolation,
  });

  final LineInfo lineInfo;
  final int maxLines;
  final void Function(String message) onViolation;

  @override
  void visitFunctionDeclaration(FunctionDeclaration node) {
    _checkBody(node.functionExpression.body, _nameFor(node.name.lexeme, node));
    super.visitFunctionDeclaration(node);
  }

  @override
  void visitMethodDeclaration(MethodDeclaration node) {
    _checkBody(node.body, _nameFor(node.name.lexeme, node));
    super.visitMethodDeclaration(node);
  }

  @override
  void visitConstructorDeclaration(ConstructorDeclaration node) {
    if (!node.body.isSynthetic) {
      final name = node.name?.lexeme ?? '';
      final baseName = node.returnType.name;
      final ctorName = name.isEmpty ? baseName : '$baseName.$name';
      _checkBody(node.body, _nameFor(ctorName, node));
    }
    super.visitConstructorDeclaration(node);
  }

  void _checkBody(AstNode node, String name) {
    if (node is FunctionBody && node.isSynthetic) {
      return;
    }

    final start = lineInfo.getLocation(node.offset).lineNumber;
    final end = lineInfo.getLocation(node.end).lineNumber;
    final length = end - start + 1;
    if (length > maxLines) {
      onViolation('$name has $length lines (max $maxLines)');
    }
  }

  String _nameFor(String name, AstNode node) {
    final parent = node.parent;
    if (parent is ClassDeclaration) {
      return '${parent.name.lexeme}.$name';
    }
    if (parent is MixinDeclaration) {
      return '${parent.name.lexeme}.$name';
    }
    if (parent is ExtensionDeclaration && parent.name != null) {
      return '${parent.name!.lexeme}.$name';
    }
    return name;
  }
}
