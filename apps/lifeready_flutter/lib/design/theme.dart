import 'package:flutter/material.dart';
import 'tokens.dart';

ThemeData themeFromTokens(Tokens t) {
  final scheme = ColorScheme.fromSeed(
    seedColor: t.primary,
    brightness: Brightness.dark,
    surface: t.surface,
  );

  return ThemeData(
    colorScheme: scheme,
    scaffoldBackgroundColor: t.surface,
    useMaterial3: true,
    textTheme: TextTheme(
      titleLarge: TextStyle(
        fontFamily: t.fontFamily,
        fontSize: t.sizeXl,
        fontWeight: t.weightSemibold,
        height: t.lineHeightNormal,
        color: t.text,
      ),
      titleMedium: TextStyle(
        fontFamily: t.fontFamily,
        fontSize: t.sizeLg,
        fontWeight: t.weightMedium,
        height: t.lineHeightNormal,
        color: t.text,
      ),
      bodyLarge: TextStyle(
        fontFamily: t.fontFamily,
        fontSize: t.sizeMd,
        fontWeight: t.weightRegular,
        height: t.lineHeightNormal,
        color: t.text,
      ),
      bodyMedium: TextStyle(
        fontFamily: t.fontFamily,
        fontSize: t.sizeSm,
        fontWeight: t.weightRegular,
        height: t.lineHeightNormal,
        color: t.text,
      ),
      labelLarge: TextStyle(
        fontFamily: t.fontFamily,
        fontSize: t.sizeSm,
        fontWeight: t.weightMedium,
        height: t.lineHeightTight,
        color: t.text,
      ),
    ),
    dividerColor: t.outline,
    cardTheme: CardTheme(
      color: t.surfaceAlt,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(t.rMd),
        side: BorderSide(color: t.outline),
      ),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: t.surfaceAlt,
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(t.rSm),
        borderSide: BorderSide(color: t.outline),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(t.rSm),
        borderSide: BorderSide(color: t.outline),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(t.rSm),
        borderSide: BorderSide(color: t.primary, width: 1.5),
      ),
      hintStyle: TextStyle(color: t.textMuted),
    ),
  );
}
