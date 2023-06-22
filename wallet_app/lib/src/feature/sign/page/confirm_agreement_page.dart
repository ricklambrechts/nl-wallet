import 'package:flutter/material.dart';

import '../../../util/extension/build_context_extension.dart';
import '../../common/widget/attribute/data_attribute_row.dart';
import '../../common/widget/button/confirm_buttons.dart';
import '../../common/widget/button/link_button.dart';
import '../../common/widget/placeholder_screen.dart';
import '../../common/widget/policy/policy_section.dart';
import '../../common/widget/sliver_sized_box.dart';
import '../model/sign_flow.dart';

const _kContextIllustration = 'assets/non-free/images/sign_illustration_2.png';

class ConfirmAgreementPage extends StatelessWidget {
  final VoidCallback onDeclinePressed;
  final VoidCallback onAcceptPressed;
  final SignFlow flow;

  const ConfirmAgreementPage({
    required this.onDeclinePressed,
    required this.onAcceptPressed,
    required this.flow,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      child: CustomScrollView(
        slivers: <Widget>[
          const SliverSizedBox(height: 8),
          SliverToBoxAdapter(child: _buildHeaderSection(context)),
          SliverList(delegate: _getDataAttributesDelegate()),
          SliverToBoxAdapter(child: _buildDataIncorrectButton(context)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: PolicySection(flow.policy)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverToBoxAdapter(child: _buildTrustProvider(context)),
          const SliverToBoxAdapter(child: Divider(height: 32)),
          SliverFillRemaining(
            hasScrollBody: false,
            fillOverscroll: true,
            child: Container(
              alignment: Alignment.bottomCenter,
              child: ConfirmButtons(
                onAcceptPressed: onAcceptPressed,
                acceptText: context.l10n.confirmAgreementPageConfirmCta,
                onDeclinePressed: onDeclinePressed,
                declineText: context.l10n.confirmAgreementPageCancelCta,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeaderSection(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Image.asset(
            _kContextIllustration,
            fit: BoxFit.cover,
            width: double.infinity,
          ),
          const SizedBox(height: 32),
          Text(
            context.l10n.confirmAgreementPageTitle,
            style: context.textTheme.displayMedium,
            textAlign: TextAlign.start,
          ),
        ],
      ),
    );
  }

  SliverChildBuilderDelegate _getDataAttributesDelegate() {
    return SliverChildBuilderDelegate(
      (context, index) => Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: DataAttributeRow(attribute: flow.resolvedAttributes[index]),
      ),
      childCount: flow.resolvedAttributes.length,
    );
  }

  Widget _buildDataIncorrectButton(BuildContext context) {
    return Align(
      alignment: AlignmentDirectional.centerStart,
      child: LinkButton(
        onPressed: () => PlaceholderScreen.show(context),
        child: Padding(
          padding: const EdgeInsets.only(left: 8),
          child: Text(context.l10n.confirmAgreementPageDataIncorrectCta),
        ),
      ),
    );
  }

  Widget _buildTrustProvider(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16),
      child: Row(
        children: [
          Image.asset(flow.trustProvider.logoUrl),
          const SizedBox(width: 16),
          Expanded(
            child: Text(
              context.l10n.confirmAgreementPageSignProvider(flow.trustProvider.name),
              style: context.textTheme.bodyLarge,
            ),
          )
        ],
      ),
    );
  }
}
