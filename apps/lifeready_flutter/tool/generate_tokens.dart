import 'dart:convert';
import 'dart:io';

class TokenPath {
  final List<String> segments;
  const TokenPath(this.segments);
}

void main() {
  final scriptDir = File.fromUri(Platform.script).parent;
  final inputPath = scriptDir.uri.resolve(
    '../../../packages/design-tokens/tokens.dtcg.json',
  );
  final outputPath = scriptDir.uri.resolve(
    '../lib/design/generated_tokens.dart',
  );

  final inputFile = File.fromUri(inputPath);
  if (!inputFile.existsSync()) {
    stderr.writeln('Missing tokens file: ${inputFile.path}');
    exit(2);
  }

  final jsonMap =
      json.decode(inputFile.readAsStringSync()) as Map<String, dynamic>;
  final lifeready = jsonMap['lifeready'] as Map<String, dynamic>?;
  if (lifeready == null) {
    stderr.writeln('Missing lifeready root node');
    exit(2);
  }

  String getMetaString(String key) {
    final extensions = lifeready[r'$extensions'] as Map<String, dynamic>?;
    final meta = extensions?['meta'] as Map<String, dynamic>?;
    final value = meta?[key];
    if (value == null) {
      throw StateError('Missing metadata: $key');
    }
    return value.toString();
  }

  dynamic getValue(TokenPath path) => _get(path, lifeready);

  String getStringValue(TokenPath path) => getValue(path).toString();

  double getNumberValue(TokenPath path) {
    final value = getValue(path);
    if (value is num) {
      return value.toDouble();
    }
    return double.parse(value.toString());
  }

  double getDimensionValue(TokenPath path) {
    final value = getValue(path);
    if (value is Map<String, dynamic>) {
      final unit = value['unit']?.toString();
      if (unit != 'px' && unit != 'rem') {
        throw StateError('Unsupported dimension unit: $unit');
      }
      final raw = value['value'];
      if (raw is num) {
        return raw.toDouble();
      }
      return double.parse(raw.toString());
    }
    throw StateError('Invalid dimension value: $value');
  }

  int getDurationMs(TokenPath path) {
    final value = getValue(path);
    if (value is Map<String, dynamic>) {
      final unit = value['unit']?.toString();
      final raw = value['value'];
      final numeric = raw is num
          ? raw.toDouble()
          : double.parse(raw.toString());
      if (unit == 'ms') {
        return numeric.round();
      }
      if (unit == 's') {
        return (numeric * 1000).round();
      }
      throw StateError('Unsupported duration unit: $unit');
    }
    throw StateError('Invalid duration value: $value');
  }

  String getColorHex(TokenPath path) {
    final value = getValue(path);
    if (value is Map<String, dynamic>) {
      final hex = value['hex'];
      if (hex is String) {
        return hex.toLowerCase();
      }
    }
    throw StateError(
      'Missing hex value for color token at ${path.segments.join('/')}',
    );
  }

  final buffer = StringBuffer();
  buffer.writeln("import 'package:flutter/material.dart';");
  buffer.writeln();
  buffer.writeln('class GeneratedTokens {');
  buffer.writeln("  static const String brand = '${getMetaString('brand')}';");
  buffer.writeln(
    "  static const String version = '${getMetaString('version')}';",
  );
  buffer.writeln();

  buffer.writeln(
    '  static const Color surface = Color(0xff${_hex(getColorHex(const TokenPath(['color', 'surface', r'$value'])))});',
  );
  buffer.writeln(
    '  static const Color surfaceAlt = Color(0xff${_hex(getColorHex(const TokenPath(['color', 'surfaceAlt', r'$value'])))});',
  );
  buffer.writeln(
    '  static const Color text = Color(0xff${_hex(getColorHex(const TokenPath(['color', 'text', r'$value'])))});',
  );
  buffer.writeln(
    '  static const Color textMuted = Color(0xff${_hex(getColorHex(const TokenPath(['color', 'textMuted', r'$value'])))});',
  );
  buffer.writeln(
    '  static const Color primary = Color(0xff${_hex(getColorHex(const TokenPath(['color', 'primary', r'$value'])))});',
  );
  buffer.writeln(
    '  static const Color danger = Color(0xff${_hex(getColorHex(const TokenPath(['color', 'danger', r'$value'])))});',
  );
  buffer.writeln(
    '  static const Color success = Color(0xff${_hex(getColorHex(const TokenPath(['color', 'success', r'$value'])))});',
  );
  buffer.writeln(
    '  static const Color warning = Color(0xff${_hex(getColorHex(const TokenPath(['color', 'warning', r'$value'])))});',
  );
  buffer.writeln(
    '  static const Color outline = Color(0xff${_hex(getColorHex(const TokenPath(['color', 'outline', r'$value'])))});',
  );
  buffer.writeln();

  buffer.writeln(
    '  static const double rXs = ${getDimensionValue(const TokenPath(['radius', 'xs', r'$value']))};',
  );
  buffer.writeln(
    '  static const double rSm = ${getDimensionValue(const TokenPath(['radius', 'sm', r'$value']))};',
  );
  buffer.writeln(
    '  static const double rMd = ${getDimensionValue(const TokenPath(['radius', 'md', r'$value']))};',
  );
  buffer.writeln(
    '  static const double rLg = ${getDimensionValue(const TokenPath(['radius', 'lg', r'$value']))};',
  );
  buffer.writeln();

  buffer.writeln(
    '  static const double s1 = ${getDimensionValue(const TokenPath(['space', '1', r'$value']))};',
  );
  buffer.writeln(
    '  static const double s2 = ${getDimensionValue(const TokenPath(['space', '2', r'$value']))};',
  );
  buffer.writeln(
    '  static const double s3 = ${getDimensionValue(const TokenPath(['space', '3', r'$value']))};',
  );
  buffer.writeln(
    '  static const double s4 = ${getDimensionValue(const TokenPath(['space', '4', r'$value']))};',
  );
  buffer.writeln(
    '  static const double s5 = ${getDimensionValue(const TokenPath(['space', '5', r'$value']))};',
  );
  buffer.writeln(
    '  static const double s6 = ${getDimensionValue(const TokenPath(['space', '6', r'$value']))};',
  );
  buffer.writeln();

  buffer.writeln(
    "  static const String fontFamily = '${getStringValue(const TokenPath(['type', 'fontFamily', r'$value']))}';",
  );
  buffer.writeln(
    '  static const double sizeXs = ${getDimensionValue(const TokenPath(['type', 'size', 'xs', r'$value']))};',
  );
  buffer.writeln(
    '  static const double sizeSm = ${getDimensionValue(const TokenPath(['type', 'size', 'sm', r'$value']))};',
  );
  buffer.writeln(
    '  static const double sizeMd = ${getDimensionValue(const TokenPath(['type', 'size', 'md', r'$value']))};',
  );
  buffer.writeln(
    '  static const double sizeLg = ${getDimensionValue(const TokenPath(['type', 'size', 'lg', r'$value']))};',
  );
  buffer.writeln(
    '  static const double sizeXl = ${getDimensionValue(const TokenPath(['type', 'size', 'xl', r'$value']))};',
  );
  buffer.writeln(
    '  static const double size2xl = ${getDimensionValue(const TokenPath(['type', 'size', '2xl', r'$value']))};',
  );
  buffer.writeln();

  buffer.writeln(
    '  static const double weightRegular = ${getNumberValue(const TokenPath(['type', 'weight', 'regular', r'$value']))};',
  );
  buffer.writeln(
    '  static const double weightMedium = ${getNumberValue(const TokenPath(['type', 'weight', 'medium', r'$value']))};',
  );
  buffer.writeln(
    '  static const double weightSemibold = ${getNumberValue(const TokenPath(['type', 'weight', 'semibold', r'$value']))};',
  );
  buffer.writeln();

  buffer.writeln(
    '  static const double lineHeightTight = ${getNumberValue(const TokenPath(['type', 'lineHeight', 'tight', r'$value']))};',
  );
  buffer.writeln(
    '  static const double lineHeightNormal = ${getNumberValue(const TokenPath(['type', 'lineHeight', 'normal', r'$value']))};',
  );
  buffer.writeln();

  buffer.writeln(
    '  static const int motionFastMs = ${getDurationMs(const TokenPath(['motion', 'duration', 'fast', r'$value']))};',
  );
  buffer.writeln(
    '  static const int motionNormalMs = ${getDurationMs(const TokenPath(['motion', 'duration', 'normal', r'$value']))};',
  );
  buffer.writeln();

  final curve =
      _get(
            const TokenPath(['motion', 'curve', 'standard', r'$value']),
            lifeready,
          )
          as List;
  final curveList = curve.map((e) => double.parse(e.toString())).toList();
  buffer.writeln(
    '  static const List<double> curveStandard = ${_listLiteral(curveList)};',
  );
  buffer.writeln('}');

  File.fromUri(outputPath).writeAsStringSync(buffer.toString());
  stdout.writeln('Generated: ${outputPath.toFilePath()}');
}

dynamic _get(TokenPath path, Map<String, dynamic> root) {
  dynamic cur = root;
  for (final segment in path.segments) {
    if (cur is Map<String, dynamic>) {
      if (!cur.containsKey(segment)) {
        throw StateError('Missing token: ${path.segments.join('/')}');
      }
      cur = cur[segment];
      continue;
    }
    throw StateError('Invalid token path: ${path.segments.join('/')}');
  }
  return cur;
}

String _hex(String color) {
  final cleaned = color.replaceFirst('#', '').toLowerCase();
  if (cleaned.length != 6) {
    throw StateError('Invalid color token: $color');
  }
  return cleaned;
}

String _listLiteral(List<double> values) {
  final inner = values.map((value) => value.toStringAsFixed(2)).join(', ');
  return '[$inner]';
}
