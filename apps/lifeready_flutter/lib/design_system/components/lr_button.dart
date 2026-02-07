import 'package:flutter/material.dart';
import '../../design/tokens_scope.dart';

enum LrButtonVariant { primary, outline }

class LrButton extends StatelessWidget {
  final String label;
  final VoidCallback? onPressed;
  final LrButtonVariant variant;

  const LrButton({
    super.key,
    required this.label,
    this.onPressed,
    this.variant = LrButtonVariant.primary,
  });

  @override
  Widget build(BuildContext context) {
    final t = TokensScope.of(context);

    final style = ButtonStyle(
      padding: WidgetStateProperty.all(
        EdgeInsets.symmetric(horizontal: t.s5, vertical: t.s3),
      ),
      shape: WidgetStateProperty.all(
        RoundedRectangleBorder(borderRadius: BorderRadius.circular(t.rMd)),
      ),
    );

    if (variant == LrButtonVariant.outline) {
      return OutlinedButton(
        style: style.copyWith(
          side: WidgetStateProperty.all(BorderSide(color: t.outline)),
          foregroundColor: WidgetStateProperty.all(t.text),
        ),
        onPressed: onPressed,
        child: Text(label),
      );
    }

    return FilledButton(
      style: style.copyWith(
        backgroundColor: WidgetStateProperty.all(t.primary),
        foregroundColor: WidgetStateProperty.all(t.text),
      ),
      onPressed: onPressed,
      child: Text(label),
    );
  }
}
