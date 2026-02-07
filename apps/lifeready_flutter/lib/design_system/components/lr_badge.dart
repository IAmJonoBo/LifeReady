import 'package:flutter/material.dart';
import '../../design/tokens_scope.dart';

enum LrBadgeTone { primary, success, warning, danger }

class LrBadge extends StatelessWidget {
  final String label;
  final LrBadgeTone tone;

  const LrBadge({
    super.key,
    required this.label,
    this.tone = LrBadgeTone.primary,
  });

  @override
  Widget build(BuildContext context) {
    final t = TokensScope.of(context);
    final color = switch (tone) {
      LrBadgeTone.primary => t.primary,
      LrBadgeTone.success => t.success,
      LrBadgeTone.warning => t.warning,
      LrBadgeTone.danger => t.danger,
    };

    return Container(
      padding: EdgeInsets.symmetric(horizontal: t.s3, vertical: t.s2),
      decoration: BoxDecoration(
        color: color.withOpacity(0.2),
        borderRadius: BorderRadius.circular(t.rSm),
        border: Border.all(color: color),
      ),
      child: Text(
        label,
        style: TextStyle(color: color, fontWeight: FontWeight.w600),
      ),
    );
  }
}
