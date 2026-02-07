import 'package:flutter/material.dart';
import 'generated_tokens.dart';

class Tokens {
  const Tokens();

  Color get primary => GeneratedTokens.primary;
  Color get surface => GeneratedTokens.surface;
  Color get surfaceAlt => GeneratedTokens.surfaceAlt;
  Color get text => GeneratedTokens.text;
  Color get textMuted => GeneratedTokens.textMuted;
  Color get danger => GeneratedTokens.danger;
  Color get success => GeneratedTokens.success;
  Color get warning => GeneratedTokens.warning;
  Color get outline => GeneratedTokens.outline;

  double get rXs => GeneratedTokens.rXs;
  double get rSm => GeneratedTokens.rSm;
  double get rMd => GeneratedTokens.rMd;
  double get rLg => GeneratedTokens.rLg;

  double get s1 => GeneratedTokens.s1;
  double get s2 => GeneratedTokens.s2;
  double get s3 => GeneratedTokens.s3;
  double get s4 => GeneratedTokens.s4;
  double get s5 => GeneratedTokens.s5;
  double get s6 => GeneratedTokens.s6;

  String get fontFamily => GeneratedTokens.fontFamily;
  double get sizeXs => GeneratedTokens.sizeXs;
  double get sizeSm => GeneratedTokens.sizeSm;
  double get sizeMd => GeneratedTokens.sizeMd;
  double get sizeLg => GeneratedTokens.sizeLg;
  double get sizeXl => GeneratedTokens.sizeXl;
  double get size2xl => GeneratedTokens.size2xl;

  FontWeight get weightRegular => _weight(GeneratedTokens.weightRegular);
  FontWeight get weightMedium => _weight(GeneratedTokens.weightMedium);
  FontWeight get weightSemibold => _weight(GeneratedTokens.weightSemibold);

  double get lineHeightTight => GeneratedTokens.lineHeightTight;
  double get lineHeightNormal => GeneratedTokens.lineHeightNormal;

  Duration get motionFast => Duration(milliseconds: GeneratedTokens.motionFastMs);
  Duration get motionNormal => Duration(milliseconds: GeneratedTokens.motionNormalMs);

  Curve get curveStandard => Cubic(
        GeneratedTokens.curveStandard[0],
        GeneratedTokens.curveStandard[1],
        GeneratedTokens.curveStandard[2],
        GeneratedTokens.curveStandard[3],
      );

  FontWeight _weight(double value) {
    return FontWeight.values[((value / 100).round() - 1).clamp(0, 8)];
  }

  static Future<Tokens> load() async => const Tokens();
}
