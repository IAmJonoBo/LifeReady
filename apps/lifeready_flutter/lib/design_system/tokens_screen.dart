import 'package:flutter/material.dart';
import '../design/tokens_scope.dart';

class TokensScreen extends StatelessWidget {
  const TokensScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final t = TokensScope.of(context);

    final colorEntries = [
      _ColorEntry('Surface', t.surface),
      _ColorEntry('Surface Alt', t.surfaceAlt),
      _ColorEntry('Text', t.text),
      _ColorEntry('Text Muted', t.textMuted),
      _ColorEntry('Primary', t.primary),
      _ColorEntry('Success', t.success),
      _ColorEntry('Warning', t.warning),
      _ColorEntry('Danger', t.danger),
      _ColorEntry('Outline', t.outline),
    ];

    final typeEntries = [
      _TypeEntry('XS', t.sizeXs, t.weightRegular),
      _TypeEntry('SM', t.sizeSm, t.weightRegular),
      _TypeEntry('MD', t.sizeMd, t.weightRegular),
      _TypeEntry('LG', t.sizeLg, t.weightMedium),
      _TypeEntry('XL', t.sizeXl, t.weightSemibold),
      _TypeEntry('2XL', t.size2xl, t.weightSemibold),
    ];

    return Scaffold(
      appBar: AppBar(
        title: const Text('Tokens'),
        backgroundColor: t.surface,
      ),
      body: ListView(
        padding: EdgeInsets.all(t.s5),
        children: [
          _sectionTitle(context, 'Color'),
          Wrap(
            spacing: t.s3,
            runSpacing: t.s3,
            children: colorEntries
                .map((entry) => _ColorSwatch(entry: entry))
                .toList(),
          ),
          SizedBox(height: t.s6),
          _sectionTitle(context, 'Typography'),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: typeEntries
                .map((entry) => Padding(
                      padding: EdgeInsets.only(bottom: t.s3),
                      child: Text(
                        '${entry.label} ${entry.size.toStringAsFixed(0)}',
                        style: TextStyle(
                          fontFamily: t.fontFamily,
                          fontSize: entry.size,
                          fontWeight: entry.weight,
                          height: t.lineHeightNormal,
                          color: t.text,
                        ),
                      ),
                    ))
                .toList(),
          ),
          SizedBox(height: t.s6),
          _sectionTitle(context, 'Spacing'),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _spacingRow(context, 'S1', t.s1),
              _spacingRow(context, 'S2', t.s2),
              _spacingRow(context, 'S3', t.s3),
              _spacingRow(context, 'S4', t.s4),
              _spacingRow(context, 'S5', t.s5),
              _spacingRow(context, 'S6', t.s6),
            ],
          ),
        ],
      ),
    );
  }

  Widget _sectionTitle(BuildContext context, String title) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Text(title, style: Theme.of(context).textTheme.titleLarge),
    );
  }

  Widget _spacingRow(BuildContext context, String label, double value) {
    final t = TokensScope.of(context);
    return Padding(
      padding: EdgeInsets.only(bottom: t.s2),
      child: Row(
        children: [
          SizedBox(width: 50, child: Text(label, style: Theme.of(context).textTheme.bodyMedium)),
          Container(height: 12, width: value, color: t.primary),
          SizedBox(width: t.s3),
          Text(value.toStringAsFixed(0), style: Theme.of(context).textTheme.bodyMedium),
        ],
      ),
    );
  }
}

class _ColorEntry {
  final String label;
  final Color color;

  const _ColorEntry(this.label, this.color);
}

class _TypeEntry {
  final String label;
  final double size;
  final FontWeight weight;

  const _TypeEntry(this.label, this.size, this.weight);
}

class _ColorSwatch extends StatelessWidget {
  final _ColorEntry entry;

  const _ColorSwatch({required this.entry});

  @override
  Widget build(BuildContext context) {
    final t = TokensScope.of(context);
    return Container(
      width: 140,
      padding: EdgeInsets.all(t.s3),
      decoration: BoxDecoration(
        color: t.surfaceAlt,
        borderRadius: BorderRadius.circular(t.rSm),
        border: Border.all(color: t.outline),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            height: 42,
            decoration: BoxDecoration(
              color: entry.color,
              borderRadius: BorderRadius.circular(t.rXs),
            ),
          ),
          SizedBox(height: t.s2),
          Text(entry.label, style: Theme.of(context).textTheme.bodyMedium),
        ],
      ),
    );
  }
}
