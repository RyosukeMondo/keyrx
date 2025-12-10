/// Reusable styled text field widgets with unified styling.
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Standard styled text field with consistent styling across the app.
///
/// Supports common variants:
/// - Simple labeled input
/// - Bordered input with outline
/// - Multiline expanding input for code/text editors
/// - Number input with validation
class StyledTextField extends StatelessWidget {
  const StyledTextField({
    super.key,
    this.controller,
    this.labelText,
    this.hintText,
    this.suffixText,
    this.prefixIcon,
    this.suffixIcon,
    this.enabled = true,
    this.maxLines = 1,
    this.expands = false,
    this.keyboardType,
    this.inputFormatters,
    this.onChanged,
    this.onSubmitted,
    this.textAlign = TextAlign.start,
    this.textAlignVertical,
    this.style,
    this.decoration,
    this.autofocus = false,
    this.obscureText = false,
    this.autocorrect = true,
    this.focusNode,
  });

  final TextEditingController? controller;
  final String? labelText;
  final String? hintText;
  final String? suffixText;
  final Widget? prefixIcon;
  final Widget? suffixIcon;
  final bool enabled;
  final int? maxLines;
  final bool expands;
  final TextInputType? keyboardType;
  final List<TextInputFormatter>? inputFormatters;
  final ValueChanged<String>? onChanged;
  final ValueChanged<String>? onSubmitted;
  final TextAlign textAlign;
  final TextAlignVertical? textAlignVertical;
  final TextStyle? style;
  final InputDecoration? decoration;
  final bool autofocus;
  final bool obscureText;
  final bool autocorrect;
  final FocusNode? focusNode;

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      enabled: enabled,
      maxLines: maxLines,
      expands: expands,
      keyboardType: keyboardType,
      inputFormatters: inputFormatters,
      onChanged: onChanged,
      onSubmitted: onSubmitted,
      textAlign: textAlign,
      textAlignVertical: textAlignVertical,
      style: style,
      autofocus: autofocus,
      obscureText: obscureText,
      autocorrect: autocorrect,
      focusNode: focusNode,
      decoration:
          decoration ??
          InputDecoration(
            labelText: labelText,
            hintText: hintText,
            suffixText: suffixText,
            prefixIcon: prefixIcon,
            suffixIcon: suffixIcon,
          ),
    );
  }
}

/// Text field with outline border, commonly used for forms.
class OutlinedTextField extends StatelessWidget {
  const OutlinedTextField({
    super.key,
    this.controller,
    this.labelText,
    this.hintText,
    this.suffixText,
    this.prefixIcon,
    this.suffixIcon,
    this.enabled = true,
    this.maxLines = 1,
    this.keyboardType,
    this.inputFormatters,
    this.onChanged,
    this.onSubmitted,
    this.style,
    this.autofocus = false,
    this.obscureText = false,
    this.autocorrect = true,
    this.focusNode,
  });

  final TextEditingController? controller;
  final String? labelText;
  final String? hintText;
  final String? suffixText;
  final Widget? prefixIcon;
  final Widget? suffixIcon;
  final bool enabled;
  final int? maxLines;
  final TextInputType? keyboardType;
  final List<TextInputFormatter>? inputFormatters;
  final ValueChanged<String>? onChanged;
  final ValueChanged<String>? onSubmitted;
  final TextStyle? style;
  final bool autofocus;
  final bool obscureText;
  final bool autocorrect;
  final FocusNode? focusNode;

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      enabled: enabled,
      maxLines: maxLines,
      keyboardType: keyboardType,
      inputFormatters: inputFormatters,
      onChanged: onChanged,
      onSubmitted: onSubmitted,
      style: style,
      autofocus: autofocus,
      obscureText: obscureText,
      autocorrect: autocorrect,
      focusNode: focusNode,
      decoration: InputDecoration(
        labelText: labelText,
        hintText: hintText,
        suffixText: suffixText,
        prefixIcon: prefixIcon,
        suffixIcon: suffixIcon,
        border: const OutlineInputBorder(),
      ),
    );
  }
}

/// Multiline text field that expands to fill available space.
/// Commonly used for code editors or large text input.
class ExpandingTextField extends StatelessWidget {
  const ExpandingTextField({
    super.key,
    this.controller,
    this.hintText,
    this.onChanged,
    this.style,
    this.backgroundColor,
    this.padding = const EdgeInsets.all(16),
    this.autofocus = false,
    this.enabled = true,
    this.focusNode,
  });

  final TextEditingController? controller;
  final String? hintText;
  final ValueChanged<String>? onChanged;
  final TextStyle? style;
  final Color? backgroundColor;
  final EdgeInsetsGeometry padding;
  final bool autofocus;
  final bool enabled;
  final FocusNode? focusNode;

  @override
  Widget build(BuildContext context) {
    final bgColor =
        backgroundColor ?? Theme.of(context).colorScheme.surfaceContainerLowest;
    final textStyle =
        style ?? const TextStyle(fontFamily: 'monospace', fontSize: 13);

    return Container(
      color: bgColor,
      child: TextField(
        controller: controller,
        enabled: enabled,
        maxLines: null,
        expands: true,
        textAlignVertical: TextAlignVertical.top,
        style: textStyle,
        autofocus: autofocus,
        focusNode: focusNode,
        decoration: InputDecoration(
          border: InputBorder.none,
          hintText: hintText,
          contentPadding: padding,
        ),
        onChanged: onChanged,
      ),
    );
  }
}

/// Text field specifically for number input with optional validation.
class NumberTextField extends StatelessWidget {
  const NumberTextField({
    super.key,
    this.controller,
    this.labelText,
    this.hintText,
    this.suffixText,
    this.enabled = true,
    this.onChanged,
    this.onSubmitted,
    this.allowDecimal = false,
    this.allowNegative = false,
    this.outlined = false,
    this.autofocus = false,
    this.focusNode,
  });

  final TextEditingController? controller;
  final String? labelText;
  final String? hintText;
  final String? suffixText;
  final bool enabled;
  final ValueChanged<String>? onChanged;
  final ValueChanged<String>? onSubmitted;
  final bool allowDecimal;
  final bool allowNegative;
  final bool outlined;
  final bool autofocus;
  final FocusNode? focusNode;

  @override
  Widget build(BuildContext context) {
    final inputFormatters = <TextInputFormatter>[
      if (!allowDecimal && !allowNegative)
        FilteringTextInputFormatter.digitsOnly
      else
        FilteringTextInputFormatter.allow(RegExp(_buildNumberPattern())),
    ];

    final decoration = InputDecoration(
      labelText: labelText,
      hintText: hintText,
      suffixText: suffixText,
      border: outlined ? const OutlineInputBorder() : null,
    );

    return TextField(
      controller: controller,
      enabled: enabled,
      keyboardType: TextInputType.numberWithOptions(
        decimal: allowDecimal,
        signed: allowNegative,
      ),
      inputFormatters: inputFormatters,
      onChanged: onChanged,
      onSubmitted: onSubmitted,
      decoration: decoration,
      autofocus: autofocus,
      focusNode: focusNode,
    );
  }

  String _buildNumberPattern() {
    final parts = ['0-9'];
    if (allowDecimal) parts.add('.');
    if (allowNegative) parts.add('-');
    return '[${parts.join()}]+';
  }
}
