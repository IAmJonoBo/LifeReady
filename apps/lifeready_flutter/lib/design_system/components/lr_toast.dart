import 'package:flutter/material.dart';
import '../../design/tokens_scope.dart';

class LrToast {
  static void show(BuildContext context, {required String message}) {
    final t = TokensScope.of(context);
    final snackBar = SnackBar(
      content: Text(message),
      backgroundColor: t.surfaceAlt,
      behavior: SnackBarBehavior.floating,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(t.rSm)),
    );
    ScaffoldMessenger.of(context).showSnackBar(snackBar);
  }
}
