import 'package:flutter/material.dart';
import '../../design/tokens_scope.dart';

enum LrTimelineState { complete, current, upcoming }

class LrTimelineStep extends StatelessWidget {
  final String title;
  final String subtitle;
  final LrTimelineState state;

  const LrTimelineStep({
    super.key,
    required this.title,
    required this.subtitle,
    this.state = LrTimelineState.upcoming,
  });

  @override
  Widget build(BuildContext context) {
    final t = TokensScope.of(context);
    final color = switch (state) {
      LrTimelineState.complete => t.success,
      LrTimelineState.current => t.primary,
      LrTimelineState.upcoming => t.outline,
    };

    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Column(
          children: [
            Container(
              width: 14,
              height: 14,
              decoration: BoxDecoration(color: color, shape: BoxShape.circle),
            ),
            Container(
              width: 2,
              height: 36,
              color: color.withValues(alpha: 0.4),
            ),
          ],
        ),
        SizedBox(width: t.s4),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(title, style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 4),
              Text(subtitle, style: Theme.of(context).textTheme.bodyMedium),
            ],
          ),
        ),
      ],
    );
  }
}
