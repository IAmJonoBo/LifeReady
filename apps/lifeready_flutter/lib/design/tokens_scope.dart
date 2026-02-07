import 'package:flutter/widgets.dart';
import 'tokens.dart';

class TokensScope extends InheritedWidget {
  final Tokens tokens;

  const TokensScope({super.key, required this.tokens, required super.child});

  static Tokens of(BuildContext context) {
    final scope = context.dependOnInheritedWidgetOfExactType<TokensScope>();
    if (scope == null) {
      throw FlutterError('TokensScope not found in widget tree');
    }
    return scope.tokens;
  }

  @override
  bool updateShouldNotify(TokensScope oldWidget) => tokens != oldWidget.tokens;
}
