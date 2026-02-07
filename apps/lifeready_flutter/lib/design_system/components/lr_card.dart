import 'package:flutter/material.dart';
import '../../design/tokens_scope.dart';

class LrCard extends StatelessWidget {
  final Widget child;
  final EdgeInsetsGeometry? padding;

  const LrCard({
    super.key,
    required this.child,
    this.padding,
  });

  @override
  Widget build(BuildContext context) {
    final t = TokensScope.of(context);
    return Container(
      padding: padding ?? EdgeInsets.all(t.s4),
      decoration: BoxDecoration(
        color: t.surfaceAlt,
        borderRadius: BorderRadius.circular(t.rMd),
        border: Border.all(color: t.outline),
      ),
      child: child,
    );
  }
}
