import 'package:flutter/material.dart';
import 'design/tokens.dart';
import 'design/tokens_scope.dart';
import 'design/theme.dart';
import 'design_system/design_system_screen.dart';
import 'design_system/tokens_screen.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  runApp(const LifeReadyApp());
}

class LifeReadyApp extends StatefulWidget {
  const LifeReadyApp({super.key});

  @override
  State<LifeReadyApp> createState() => _LifeReadyAppState();
}

class _LifeReadyAppState extends State<LifeReadyApp> {
  late final Future<Tokens> _tokens = Tokens.load();

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<Tokens>(
      future: _tokens,
      builder: (context, snapshot) {
        if (!snapshot.hasData) {
          return const MaterialApp(
            home: Scaffold(body: Center(child: CircularProgressIndicator())),
          );
        }
        final tokens = snapshot.data!;
        return TokensScope(
          tokens: tokens,
          child: MaterialApp(
            title: 'LifeReady',
            theme: themeFromTokens(tokens),
            routes: {
              '/': (_) => const DesignSystemScreen(),
              '/design-system': (_) => const DesignSystemScreen(),
              '/tokens': (_) => const TokensScreen(),
            },
          ),
        );
      },
    );
  }
}
