import 'package:flutter/material.dart';

class LrInput extends StatelessWidget {
  final String label;
  final String? hint;

  const LrInput({
    super.key,
    required this.label,
    this.hint,
  });

  @override
  Widget build(BuildContext context) {
    return TextField(
      decoration: InputDecoration(
        labelText: label,
        hintText: hint,
      ),
    );
  }
}
