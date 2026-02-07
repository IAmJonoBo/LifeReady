import 'dart:convert';
import 'dart:io';

class TokenPath {
  final List<String> segments;
  const TokenPath(this.segments);
}

void main() {
  final scriptDir = File.fromUri(Platform.script).parent;
  final inputPath = scriptDir.uri.resolve('../../packages/design-tokens/tokens.dtcg.json');
  final outputPath = scriptDir.uri.resolve('../lib/design/generated_tokens.dart');

  final inputFile = File.fromUri(inputPath);
  if (!inputFile.existsSync()) {
    stderr.writeln('Missing tokens file: ${inputFile.path}');
    exit(2);
  }

  final jsonMap = json.decode(inputFile.readAsStringSync()) as Map<String, dynamic>;
  final lifeready = jsonMap['lifeready'] as Map<String, dynamic>?;
  if (lifeready == null) {
    stderr.writeln('Missing lifeready root node');
    exit(2);
  }

  String getString(TokenPath path) => _get(path, lifeready).toString();
  double getDouble(TokenPath path) => double.parse(getString(path));
  String getColor(TokenPath path) => getString(path).toLowerCase();

  final buffer = StringBuffer();
  buffer.writeln("import 'package:flutter/material.dart';");
  buffer.writeln();
  buffer.writeln('class GeneratedTokens {');
  buffer.writeln("  static const String brand = '${getString(const TokenPath(['meta', 'brand', r'\$value']))}';");
  buffer.writeln("  static const String version = '${getString(const TokenPath(['meta', 'version', r'\$value']))}';");
  buffer.writeln();

  buffer.writeln('  static const Color surface = Color(0xff${_hex(getColor(const TokenPath(['color', 'surface', r'\$value'])))});');
  buffer.writeln('  static const Color surfaceAlt = Color(0xff${_hex(getColor(const TokenPath(['color', 'surfaceAlt', r'\$value'])))});');
  buffer.writeln('  static const Color text = Color(0xff${_hex(getColor(const TokenPath(['color', 'text', r'\$value'])))});');
  buffer.writeln('  static const Color textMuted = Color(0xff${_hex(getColor(const TokenPath(['color', 'textMuted', r'\$value'])))});');
  buffer.writeln('  static const Color primary = Color(0xff${_hex(getColor(const TokenPath(['color', 'primary', r'\$value'])))});');
  buffer.writeln('  static const Color danger = Color(0xff${_hex(getColor(const TokenPath(['color', 'danger', r'\$value'])))});');
  buffer.writeln('  static const Color success = Color(0xff${_hex(getColor(const TokenPath(['color', 'success', r'\$value'])))});');
  buffer.writeln('  static const Color warning = Color(0xff${_hex(getColor(const TokenPath(['color', 'warning', r'\$value'])))});');
  buffer.writeln('  static const Color outline = Color(0xff${_hex(getColor(const TokenPath(['color', 'outline', r'\$value'])))});');
  buffer.writeln();

  buffer.writeln('  static const double rXs = ${getDouble(const TokenPath(['radius', 'xs', r'\$value']))};');
  buffer.writeln('  static const double rSm = ${getDouble(const TokenPath(['radius', 'sm', r'\$value']))};');
  buffer.writeln('  static const double rMd = ${getDouble(const TokenPath(['radius', 'md', r'\$value']))};');
  buffer.writeln('  static const double rLg = ${getDouble(const TokenPath(['radius', 'lg', r'\$value']))};');
  buffer.writeln();

  buffer.writeln('  static const double s1 = ${getDouble(const TokenPath(['space', '1', r'\$value']))};');
  buffer.writeln('  static const double s2 = ${getDouble(const TokenPath(['space', '2', r'\$value']))};');
  buffer.writeln('  static const double s3 = ${getDouble(const TokenPath(['space', '3', r'\$value']))};');
  buffer.writeln('  static const double s4 = ${getDouble(const TokenPath(['space', '4', r'\$value']))};');
  buffer.writeln('  static const double s5 = ${getDouble(const TokenPath(['space', '5', r'\$value']))};');
  buffer.writeln('  static const double s6 = ${getDouble(const TokenPath(['space', '6', r'\$value']))};');
  buffer.writeln();

  buffer.writeln("  static const String fontFamily = '${getString(const TokenPath(['type', 'fontFamily', r'\$value']))}';");
  buffer.writeln('  static const double sizeXs = ${getDouble(const TokenPath(['type', 'size', 'xs', r'\$value']))};');
  buffer.writeln('  static const double sizeSm = ${getDouble(const TokenPath(['type', 'size', 'sm', r'\$value']))};');
  buffer.writeln('  static const double sizeMd = ${getDouble(const TokenPath(['type', 'size', 'md', r'\$value']))};');
  buffer.writeln('  static const double sizeLg = ${getDouble(const TokenPath(['type', 'size', 'lg', r'\$value']))};');
  buffer.writeln('  static const double sizeXl = ${getDouble(const TokenPath(['type', 'size', 'xl', r'\$value']))};');
  buffer.writeln('  static const double size2xl = ${getDouble(const TokenPath(['type', 'size', '2xl', r'\$value']))};');
  buffer.writeln();

  buffer.writeln('  static const double weightRegular = ${getDouble(const TokenPath(['type', 'weight', 'regular', r'\$value']))};');
  buffer.writeln('  static const double weightMedium = ${getDouble(const TokenPath(['type', 'weight', 'medium', r'\$value']))};');
  buffer.writeln('  static const double weightSemibold = ${getDouble(const TokenPath(['type', 'weight', 'semibold', r'\$value']))};');
  buffer.writeln();

  buffer.writeln('  static const double lineHeightTight = ${getDouble(const TokenPath(['type', 'lineHeight', 'tight', r'\$value']))};');
  buffer.writeln('  static const double lineHeightNormal = ${getDouble(const TokenPath(['type', 'lineHeight', 'normal', r'\$value']))};');
  buffer.writeln();

  buffer.writeln('  static const int motionFastMs = ${getDouble(const TokenPath(['motion', 'duration', 'fast', r'\$value']))}.round();');
  buffer.writeln('  static const int motionNormalMs = ${getDouble(const TokenPath(['motion', 'duration', 'normal', r'\$value']))}.round();');
  buffer.writeln();

  final curve = _get(const TokenPath(['motion', 'curve', 'standard', r'\$value']), lifeready) as List;
  final curveList = curve.map((e) => double.parse(e.toString())).toList();
  buffer.writeln('  static const List<double> curveStandard = ${_listLiteral(curveList)};');
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
