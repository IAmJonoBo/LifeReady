import 'package:flutter/material.dart';
import '../design/tokens_scope.dart';
import 'components/lr_badge.dart';
import 'components/lr_button.dart';
import 'components/lr_card.dart';
import 'components/lr_input.dart';
import 'components/lr_sheet.dart';
import 'components/lr_toast.dart';
import 'components/lr_timeline_step.dart';
import 'tokens_screen.dart';

class DesignSystemScreen extends StatelessWidget {
  const DesignSystemScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final t = TokensScope.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Design System'),
        backgroundColor: t.surface,
      ),
      body: ListView(
        padding: EdgeInsets.all(t.s5),
        children: [
          LrCard(
            padding: EdgeInsets.all(t.s4),
            child: Row(
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Tokens',
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                      SizedBox(height: t.s2),
                      Text(
                        'Browse colors, type scale, and spacing.',
                        style: Theme.of(context).textTheme.bodyMedium,
                      ),
                    ],
                  ),
                ),
                LrButton(
                  label: 'View',
                  onPressed: () {
                    Navigator.push(
                      context,
                      MaterialPageRoute(builder: (_) => const TokensScreen()),
                    );
                  },
                ),
              ],
            ),
          ),
          SizedBox(height: t.s5),
          _sectionTitle(context, 'Buttons'),
          Wrap(
            spacing: t.s3,
            runSpacing: t.s3,
            children: [
              LrButton(label: 'Primary', onPressed: () {}),
              LrButton(
                label: 'Outline',
                variant: LrButtonVariant.outline,
                onPressed: () {},
              ),
            ],
          ),
          SizedBox(height: t.s5),
          _sectionTitle(context, 'Inputs'),
          const LrInput(label: 'Full name', hint: 'Jane Doe'),
          SizedBox(height: t.s5),
          _sectionTitle(context, 'Cards'),
          LrCard(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Emergency Pack',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                SizedBox(height: t.s2),
                Text(
                  'Quick access bundle with directives and contacts.',
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
              ],
            ),
          ),
          SizedBox(height: t.s5),
          _sectionTitle(context, 'Badges'),
          Wrap(
            spacing: t.s3,
            runSpacing: t.s3,
            children: const [
              LrBadge(label: 'Primary'),
              LrBadge(label: 'Success', tone: LrBadgeTone.success),
              LrBadge(label: 'Warning', tone: LrBadgeTone.warning),
              LrBadge(label: 'Danger', tone: LrBadgeTone.danger),
            ],
          ),
          SizedBox(height: t.s5),
          _sectionTitle(context, 'Sheet + Toast'),
          Wrap(
            spacing: t.s3,
            children: [
              LrButton(
                label: 'Show Sheet',
                onPressed: () {
                  LrSheet.show(
                    context,
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'Export Pack',
                          style: Theme.of(context).textTheme.titleMedium,
                        ),
                        SizedBox(height: t.s2),
                        Text('Ready to export the submission bundle?'),
                        SizedBox(height: t.s4),
                        LrButton(
                          label: 'Confirm',
                          onPressed: () => Navigator.pop(context),
                        ),
                      ],
                    ),
                  );
                },
              ),
              LrButton(
                label: 'Show Toast',
                variant: LrButtonVariant.outline,
                onPressed: () =>
                    LrToast.show(context, message: 'Audit proof generated.'),
              ),
            ],
          ),
          SizedBox(height: t.s5),
          _sectionTitle(context, 'Timeline'),
          const LrTimelineStep(
            title: 'Evidence collected',
            subtitle: 'All mandatory items added.',
            state: LrTimelineState.complete,
          ),
          const LrTimelineStep(
            title: 'Draft generated',
            subtitle: 'MHCA39 draft is ready for review.',
            state: LrTimelineState.current,
          ),
          const LrTimelineStep(
            title: 'Submission pack',
            subtitle: 'Awaiting applicant oath.',
            state: LrTimelineState.upcoming,
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
}
