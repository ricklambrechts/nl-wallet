import 'package:bloc_test/bloc_test.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:golden_toolkit/golden_toolkit.dart';
import 'package:wallet/src/domain/model/attribute/attribute.dart';
import 'package:wallet/src/feature/organization/detail/bloc/organization_detail_bloc.dart';
import 'package:wallet/src/feature/organization/detail/organization_detail_screen.dart';

import '../../../../wallet_app_test_widget.dart';
import '../../../mocks/mock_data.dart';
import '../../../util/device_utils.dart';
import '../../../util/test_utils.dart';

class MockOrganizationDetailBloc extends MockBloc<OrganizationDetailEvent, OrganizationDetailState>
    implements OrganizationDetailBloc {}

void main() {
  group('goldens', () {
    testGoldens('OrganizationDetailSuccess light', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: OrganizationDetailScreen(onReportIssuePressed: () {})
                .withState<OrganizationDetailBloc, OrganizationDetailState>(
              MockOrganizationDetailBloc(),
              OrganizationDetailSuccess(
                organization: WalletMockData.organization,
                isFirstInteractionWithOrganization: true,
              ),
            ),
          ),
        wrapper: walletAppWrapper(),
      );
      await screenMatchesGolden(tester, 'success.light');
    });

    testGoldens('OrganizationDetailSuccess dark', (tester) async {
      await tester.pumpDeviceBuilder(
        DeviceUtils.deviceBuilderWithPrimaryScrollController
          ..addScenario(
            widget: OrganizationDetailScreen(onReportIssuePressed: () {})
                .withState<OrganizationDetailBloc, OrganizationDetailState>(
              MockOrganizationDetailBloc(),
              OrganizationDetailSuccess(
                organization: WalletMockData.organization,
                isFirstInteractionWithOrganization: true,
              ),
            ),
          ),
        wrapper: walletAppWrapper(brightness: Brightness.dark),
      );
      await screenMatchesGolden(tester, 'success.dark');
    });

    testGoldens('OrganizationDetailInitial light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailInitial(),
        ),
      );
      await screenMatchesGolden(tester, 'loading.light');
    });

    testGoldens('OrganizationDetailFailure light', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          const OrganizationDetailFailure(organizationId: 'id'),
        ),
      );
      await screenMatchesGolden(tester, 'error.light');
    });
  });

  group('widgets', () {
    testWidgets('organization details are visible', (tester) async {
      await tester.pumpWidgetWithAppWrapper(
        const OrganizationDetailScreen().withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.organization,
            isFirstInteractionWithOrganization: true,
          ),
        ),
      );

      expect(find.text(WalletMockData.organization.legalName.testValue), findsOneWidget);
      expect(find.text(WalletMockData.organization.description!.testValue), findsOneWidget);
    });

    testWidgets('onReportIssuePressed callback is triggered when button is clicked', (tester) async {
      bool isCalled = false;
      await tester.pumpWidgetWithAppWrapper(
        OrganizationDetailScreen(
          onReportIssuePressed: () => isCalled = true,
        ).withState<OrganizationDetailBloc, OrganizationDetailState>(
          MockOrganizationDetailBloc(),
          OrganizationDetailSuccess(
            organization: WalletMockData.organization,
            isFirstInteractionWithOrganization: true,
          ),
        ),
      );

      final l10n = await TestUtils.englishLocalizations;
      final reportIssueButtonFinder = find.text(l10n.organizationDetailScreenReportIssueCta);
      expect(reportIssueButtonFinder, findsOneWidget);
      await tester.tap(reportIssueButtonFinder);
      expect(isCalled, isTrue);
    });
  });
}
