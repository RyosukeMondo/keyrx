import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../../services/api_docs_service.dart';

/// Widget for browsing and searching Rhai API documentation.
///
/// Provides a searchable, browsable interface for viewing API documentation
/// including functions, types, parameters, and code examples with syntax
/// highlighting.
class ApiBrowser extends StatefulWidget {
  final ApiDocsService docsService;
  final String? initialModule;
  final String? initialFunction;

  const ApiBrowser({
    super.key,
    required this.docsService,
    this.initialModule,
    this.initialFunction,
  });

  @override
  State<ApiBrowser> createState() => _ApiBrowserState();
}

class _ApiBrowserState extends State<ApiBrowser> {
  final TextEditingController _searchController = TextEditingController();
  final FocusNode _searchFocusNode = FocusNode();

  String? _selectedModule;
  Object? _selectedItem; // FunctionDoc or TypeDoc
  List<SearchResult> _searchResults = [];
  bool _isSearching = false;

  @override
  void initState() {
    super.initState();
    _selectedModule = widget.initialModule;
    if (widget.initialModule != null && widget.initialFunction != null) {
      _selectedItem = widget.docsService.getFunction(
        widget.initialModule!,
        widget.initialFunction!,
      );
    }
  }

  @override
  void dispose() {
    _searchController.dispose();
    _searchFocusNode.dispose();
    super.dispose();
  }

  void _onSearchChanged(String query) {
    setState(() {
      if (query.isEmpty) {
        _isSearching = false;
        _searchResults = [];
      } else {
        _isSearching = true;
        _searchResults = widget.docsService.search(query);
      }
    });
  }

  void _onSearchResultSelected(SearchResult result) {
    setState(() {
      _selectedModule = result.module;
      _isSearching = false;
      _searchController.clear();
      _searchResults = [];

      if (result.type == 'function') {
        _selectedItem = widget.docsService.getFunction(
          result.module,
          result.name,
        );
      } else if (result.type == 'type') {
        _selectedItem = widget.docsService.getType(
          result.module,
          result.name,
        );
      } else if (result.type == 'method') {
        final parts = result.name.split('.');
        if (parts.length == 2) {
          _selectedItem = widget.docsService.getType(
            result.module,
            parts[0],
          );
        }
      }
    });
  }

  void _onModuleSelected(String? module) {
    setState(() {
      _selectedModule = module;
      _selectedItem = null;
    });
  }

  void _onFunctionSelected(FunctionDoc function) {
    setState(() {
      _selectedItem = function;
    });
  }

  void _onTypeSelected(TypeDoc type) {
    setState(() {
      _selectedItem = type;
    });
  }

  @override
  Widget build(BuildContext context) {
    if (!widget.docsService.isLoaded) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.description_outlined, size: 64, color: Colors.grey),
            SizedBox(height: 16),
            Text('No API documentation loaded'),
            SizedBox(height: 8),
            Text(
              'Run "keyrx docs generate" to generate documentation',
              style: TextStyle(color: Colors.grey, fontSize: 12),
            ),
          ],
        ),
      );
    }

    return Column(
      children: [
        _buildSearchBar(),
        Expanded(
          child: _isSearching
              ? _buildSearchResults()
              : Row(
                  children: [
                    SizedBox(
                      width: 250,
                      child: _buildModuleBrowser(),
                    ),
                    const VerticalDivider(width: 1),
                    Expanded(
                      child: _buildContentViewer(),
                    ),
                  ],
                ),
        ),
      ],
    );
  }

  Widget _buildSearchBar() {
    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        border: Border(
          bottom: BorderSide(
            color: Theme.of(context).dividerColor,
          ),
        ),
      ),
      child: TextField(
        controller: _searchController,
        focusNode: _searchFocusNode,
        decoration: InputDecoration(
          hintText: 'Search API documentation...',
          prefixIcon: const Icon(Icons.search),
          suffixIcon: _searchController.text.isNotEmpty
              ? IconButton(
                  icon: const Icon(Icons.clear),
                  onPressed: () {
                    _searchController.clear();
                    _onSearchChanged('');
                  },
                )
              : null,
          border: OutlineInputBorder(
            borderRadius: BorderRadius.circular(8),
          ),
          contentPadding: const EdgeInsets.symmetric(
            horizontal: 16,
            vertical: 12,
          ),
        ),
        onChanged: _onSearchChanged,
      ),
    );
  }

  Widget _buildSearchResults() {
    if (_searchResults.isEmpty) {
      return const Center(
        child: Text('No results found'),
      );
    }

    return ListView.builder(
      itemCount: _searchResults.length,
      itemBuilder: (context, index) {
        final result = _searchResults[index];
        return ListTile(
          leading: Icon(_getIconForType(result.type)),
          title: Text(result.name),
          subtitle: Text(
            '${result.module} \u2022 ${result.description}',
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          trailing: Text(
            '${(result.relevance * 100).toInt()}%',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          onTap: () => _onSearchResultSelected(result),
        );
      },
    );
  }

  Widget _buildModuleBrowser() {
    final modules = widget.docsService.moduleNames;

    return Column(
      children: [
        Container(
          padding: const EdgeInsets.all(12),
          alignment: Alignment.centerLeft,
          decoration: BoxDecoration(
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            border: Border(
              bottom: BorderSide(
                color: Theme.of(context).dividerColor,
              ),
            ),
          ),
          child: Text(
            'Modules',
            style: Theme.of(context).textTheme.titleMedium,
          ),
        ),
        Expanded(
          child: ListView(
            children: modules.map((moduleName) {
              final module = widget.docsService.getModule(moduleName);
              if (module == null) return const SizedBox.shrink();

              final isSelected = _selectedModule == moduleName;

              return ExpansionTile(
                leading: const Icon(Icons.folder_outlined),
                title: Text(moduleName),
                subtitle: module.description != null
                    ? Text(
                        module.description!,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      )
                    : null,
                initiallyExpanded: isSelected,
                backgroundColor: isSelected
                    ? Theme.of(context).colorScheme.primaryContainer
                    : null,
                onExpansionChanged: (expanded) {
                  if (expanded) {
                    _onModuleSelected(moduleName);
                  }
                },
                children: [
                  if (module.functions.isNotEmpty) ...[
                    Padding(
                      padding: const EdgeInsets.only(left: 32, top: 8),
                      child: Align(
                        alignment: Alignment.centerLeft,
                        child: Text(
                          'Functions',
                          style: Theme.of(context)
                              .textTheme
                              .labelSmall
                              ?.copyWith(fontWeight: FontWeight.bold),
                        ),
                      ),
                    ),
                    ...module.functions.map((func) {
                      return ListTile(
                        contentPadding: const EdgeInsets.only(left: 48),
                        leading: const Icon(
                          Icons.functions,
                          size: 18,
                        ),
                        title: Text(func.name),
                        dense: true,
                        selected: _selectedItem == func,
                        onTap: () => _onFunctionSelected(func),
                      );
                    }),
                  ],
                  if (module.types.isNotEmpty) ...[
                    Padding(
                      padding: const EdgeInsets.only(left: 32, top: 8),
                      child: Align(
                        alignment: Alignment.centerLeft,
                        child: Text(
                          'Types',
                          style: Theme.of(context)
                              .textTheme
                              .labelSmall
                              ?.copyWith(fontWeight: FontWeight.bold),
                        ),
                      ),
                    ),
                    ...module.types.map((type) {
                      return ListTile(
                        contentPadding: const EdgeInsets.only(left: 48),
                        leading: const Icon(
                          Icons.class_outlined,
                          size: 18,
                        ),
                        title: Text(type.name),
                        dense: true,
                        selected: _selectedItem == type,
                        onTap: () => _onTypeSelected(type),
                      );
                    }),
                  ],
                ],
              );
            }).toList(),
          ),
        ),
      ],
    );
  }

  Widget _buildContentViewer() {
    if (_selectedItem == null) {
      if (_selectedModule != null) {
        final module = widget.docsService.getModule(_selectedModule!);
        if (module != null) {
          return _buildModuleOverview(module);
        }
      }

      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.description_outlined,
              size: 64,
              color: Theme.of(context).colorScheme.outline,
            ),
            const SizedBox(height: 16),
            Text(
              'Select a function or type to view documentation',
              style: TextStyle(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
          ],
        ),
      );
    }

    if (_selectedItem is FunctionDoc) {
      return _buildFunctionViewer(_selectedItem as FunctionDoc);
    } else if (_selectedItem is TypeDoc) {
      return _buildTypeViewer(_selectedItem as TypeDoc);
    }

    return const SizedBox.shrink();
  }

  Widget _buildModuleOverview(ModuleDoc module) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            module.name,
            style: Theme.of(context).textTheme.headlineMedium,
          ),
          if (module.description != null) ...[
            const SizedBox(height: 8),
            Text(
              module.description!,
              style: Theme.of(context).textTheme.bodyLarge,
            ),
          ],
          const SizedBox(height: 24),
          _buildSection(
            'Functions',
            '${module.functions.length} function(s)',
            Icons.functions,
          ),
          const SizedBox(height: 16),
          _buildSection(
            'Types',
            '${module.types.length} type(s)',
            Icons.class_outlined,
          ),
        ],
      ),
    );
  }

  Widget _buildSection(String title, String subtitle, IconData icon) {
    return Card(
      child: ListTile(
        leading: Icon(icon),
        title: Text(title),
        subtitle: Text(subtitle),
      ),
    );
  }

  Widget _buildFunctionViewer(FunctionDoc function) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.functions),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  function.name,
                  style: Theme.of(context).textTheme.headlineMedium,
                ),
              ),
              if (function.deprecated != null)
                Chip(
                  label: const Text('DEPRECATED'),
                  backgroundColor: Theme.of(context).colorScheme.errorContainer,
                ),
            ],
          ),
          const SizedBox(height: 16),
          Text(
            function.description,
            style: Theme.of(context).textTheme.bodyLarge,
          ),
          if (function.deprecated != null) ...[
            const SizedBox(height: 16),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.errorContainer,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                children: [
                  Icon(
                    Icons.warning,
                    color: Theme.of(context).colorScheme.error,
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      function.deprecated!,
                      style: TextStyle(
                        color: Theme.of(context).colorScheme.onErrorContainer,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
          const SizedBox(height: 24),
          if (function.parameters.isNotEmpty) ...[
            Text(
              'Parameters',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            ...function.parameters.map((param) => _buildParameterCard(param)),
            const SizedBox(height: 24),
          ],
          if (function.returns != null) ...[
            Text(
              'Returns',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            _buildReturnCard(function.returns!),
            const SizedBox(height: 24),
          ],
          if (function.examples.isNotEmpty) ...[
            Text(
              'Examples',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            ...function.examples.map((example) => _buildCodeExample(example)),
          ],
          if (function.notes != null) ...[
            const SizedBox(height: 24),
            Text(
              'Notes',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.secondaryContainer,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Icon(
                    Icons.info_outline,
                    color: Theme.of(context).colorScheme.secondary,
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      function.notes!,
                      style: TextStyle(
                        color:
                            Theme.of(context).colorScheme.onSecondaryContainer,
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildTypeViewer(TypeDoc type) {
    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              const Icon(Icons.class_outlined),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  type.name,
                  style: Theme.of(context).textTheme.headlineMedium,
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          Text(
            type.description,
            style: Theme.of(context).textTheme.bodyLarge,
          ),
          const SizedBox(height: 24),
          if (type.properties.isNotEmpty) ...[
            Text(
              'Properties',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            ...type.properties.map((prop) => _buildPropertyCard(prop)),
            const SizedBox(height: 24),
          ],
          if (type.constructors.isNotEmpty) ...[
            Text(
              'Constructors',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            ...type.constructors.map((ctor) => _buildMethodCard(ctor)),
            const SizedBox(height: 24),
          ],
          if (type.methods.isNotEmpty) ...[
            Text(
              'Methods',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            ...type.methods.map((method) => _buildMethodCard(method)),
            const SizedBox(height: 24),
          ],
          if (type.examples.isNotEmpty) ...[
            Text(
              'Examples',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            ...type.examples.map((example) => _buildCodeExample(example)),
          ],
        ],
      ),
    );
  }

  Widget _buildParameterCard(ParameterDoc param) {
    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(
                  param.name,
                  style: const TextStyle(
                    fontFamily: 'monospace',
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(width: 8),
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 8,
                    vertical: 2,
                  ),
                  decoration: BoxDecoration(
                    color: Theme.of(context).colorScheme.secondaryContainer,
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    param.typeName,
                    style: TextStyle(
                      fontFamily: 'monospace',
                      fontSize: 12,
                      color: Theme.of(context).colorScheme.onSecondaryContainer,
                    ),
                  ),
                ),
                if (param.optional) ...[
                  const SizedBox(width: 8),
                  Chip(
                    label: const Text('optional'),
                    labelStyle: const TextStyle(fontSize: 10),
                    visualDensity: VisualDensity.compact,
                  ),
                ],
                if (param.defaultValue != null) ...[
                  const SizedBox(width: 8),
                  Text(
                    '= ${param.defaultValue}',
                    style: const TextStyle(
                      fontFamily: 'monospace',
                      fontSize: 12,
                      color: Colors.grey,
                    ),
                  ),
                ],
              ],
            ),
            const SizedBox(height: 4),
            Text(param.description),
          ],
        ),
      ),
    );
  }

  Widget _buildReturnCard(ReturnTypeDoc returns) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.primaryContainer,
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(
                returns.typeName,
                style: TextStyle(
                  fontFamily: 'monospace',
                  fontSize: 12,
                  color: Theme.of(context).colorScheme.onPrimaryContainer,
                ),
              ),
            ),
            const SizedBox(width: 8),
            Expanded(child: Text(returns.description)),
          ],
        ),
      ),
    );
  }

  Widget _buildPropertyCard(PropertyDoc prop) {
    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(
                  prop.name,
                  style: const TextStyle(
                    fontFamily: 'monospace',
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(width: 8),
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 8,
                    vertical: 2,
                  ),
                  decoration: BoxDecoration(
                    color: Theme.of(context).colorScheme.secondaryContainer,
                    borderRadius: BorderRadius.circular(4),
                  ),
                  child: Text(
                    prop.typeName,
                    style: TextStyle(
                      fontFamily: 'monospace',
                      fontSize: 12,
                      color: Theme.of(context).colorScheme.onSecondaryContainer,
                    ),
                  ),
                ),
                if (prop.readonly) ...[
                  const SizedBox(width: 8),
                  Chip(
                    label: const Text('readonly'),
                    labelStyle: const TextStyle(fontSize: 10),
                    visualDensity: VisualDensity.compact,
                  ),
                ],
              ],
            ),
            const SizedBox(height: 4),
            Text(prop.description),
          ],
        ),
      ),
    );
  }

  Widget _buildMethodCard(FunctionDoc method) {
    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: InkWell(
        onTap: () => _onFunctionSelected(method),
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  const Icon(Icons.functions, size: 16),
                  const SizedBox(width: 8),
                  Text(
                    method.name,
                    style: const TextStyle(
                      fontFamily: 'monospace',
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 4),
              Text(
                method.description,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildCodeExample(String code) {
    return Container(
      margin: const EdgeInsets.only(bottom: 12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).dividerColor,
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceContainer,
              borderRadius: const BorderRadius.only(
                topLeft: Radius.circular(8),
                topRight: Radius.circular(8),
              ),
            ),
            child: Row(
              children: [
                const Icon(Icons.code, size: 16),
                const SizedBox(width: 8),
                const Text('Example'),
                const Spacer(),
                IconButton(
                  icon: const Icon(Icons.copy, size: 16),
                  onPressed: () {
                    Clipboard.setData(ClipboardData(text: code));
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text('Code copied to clipboard'),
                        duration: Duration(seconds: 2),
                      ),
                    );
                  },
                  tooltip: 'Copy to clipboard',
                  visualDensity: VisualDensity.compact,
                ),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(12),
            child: SyntaxHighlightedCode(code: code),
          ),
        ],
      ),
    );
  }

  IconData _getIconForType(String type) {
    switch (type) {
      case 'function':
        return Icons.functions;
      case 'type':
        return Icons.class_outlined;
      case 'method':
        return Icons.functions;
      case 'property':
        return Icons.data_object;
      default:
        return Icons.description_outlined;
    }
  }
}

/// Widget for displaying syntax-highlighted Rhai code.
class SyntaxHighlightedCode extends StatelessWidget {
  final String code;

  const SyntaxHighlightedCode({
    super.key,
    required this.code,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final isDark = theme.brightness == Brightness.dark;

    return SelectableText.rich(
      TextSpan(
        style: TextStyle(
          fontFamily: 'monospace',
          fontSize: 13,
          color: theme.colorScheme.onSurface,
        ),
        children: _highlightCode(code, isDark),
      ),
    );
  }

  List<TextSpan> _highlightCode(String code, bool isDark) {
    final spans = <TextSpan>[];
    final keywords = {
      'let',
      'const',
      'fn',
      'if',
      'else',
      'for',
      'while',
      'loop',
      'break',
      'continue',
      'return',
      'true',
      'false',
      'null',
      'in',
      'import',
      'export',
      'as',
      'private',
      'this',
    };

    final keywordColor = isDark ? Colors.purple[300] : Colors.purple[700];
    final stringColor = isDark ? Colors.green[300] : Colors.green[700];
    final numberColor = isDark ? Colors.orange[300] : Colors.orange[700];
    final commentColor = isDark ? Colors.grey[500] : Colors.grey[600];
    final functionColor = isDark ? Colors.blue[300] : Colors.blue[700];

    final lines = code.split('\n');
    for (var i = 0; i < lines.length; i++) {
      final line = lines[i];
      var pos = 0;

      while (pos < line.length) {
        // Comments
        if (pos < line.length - 1 && line.substring(pos, pos + 2) == '//') {
          spans.add(
            TextSpan(
              text: line.substring(pos),
              style: TextStyle(color: commentColor, fontStyle: FontStyle.italic),
            ),
          );
          break;
        }

        // String literals
        if (line[pos] == '"' || line[pos] == "'") {
          final quote = line[pos];
          final start = pos;
          pos++;
          while (pos < line.length && line[pos] != quote) {
            if (line[pos] == '\\' && pos + 1 < line.length) {
              pos += 2;
            } else {
              pos++;
            }
          }
          if (pos < line.length) pos++; // Include closing quote
          spans.add(
            TextSpan(
              text: line.substring(start, pos),
              style: TextStyle(color: stringColor),
            ),
          );
          continue;
        }

        // Numbers
        if (RegExp(r'[0-9]').hasMatch(line[pos])) {
          final start = pos;
          while (pos < line.length &&
              RegExp(r'[0-9._]').hasMatch(line[pos])) {
            pos++;
          }
          spans.add(
            TextSpan(
              text: line.substring(start, pos),
              style: TextStyle(color: numberColor),
            ),
          );
          continue;
        }

        // Keywords and identifiers
        if (RegExp(r'[a-zA-Z_]').hasMatch(line[pos])) {
          final start = pos;
          while (pos < line.length &&
              RegExp(r'[a-zA-Z0-9_]').hasMatch(line[pos])) {
            pos++;
          }
          final word = line.substring(start, pos);

          if (keywords.contains(word)) {
            spans.add(
              TextSpan(
                text: word,
                style: TextStyle(
                  color: keywordColor,
                  fontWeight: FontWeight.bold,
                ),
              ),
            );
          } else if (pos < line.length && line[pos] == '(') {
            // Function call
            spans.add(
              TextSpan(
                text: word,
                style: TextStyle(color: functionColor),
              ),
            );
          } else {
            spans.add(TextSpan(text: word));
          }
          continue;
        }

        // Other characters
        spans.add(TextSpan(text: line[pos]));
        pos++;
      }

      // Add newline except for the last line
      if (i < lines.length - 1) {
        spans.add(const TextSpan(text: '\n'));
      }
    }

    return spans;
  }
}
