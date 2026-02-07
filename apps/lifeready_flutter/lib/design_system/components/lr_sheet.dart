import 'package:flutter/material.dart';
import '../../design/tokens_scope.dart';

class LrSheet {
  static Future<void> show(BuildContext context, {required Widget child}) {
    final t = TokensScope.of(context);
    return showModalBottomSheet<void>(
      context: context,
      backgroundColor: t.surfaceAlt,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.vertical(top: Radius.circular(t.rLg)),
      ),
      builder: (context) => Padding(
        padding: EdgeInsets.all(t.s5),
        child: child,
      ),
    );
  }
}
