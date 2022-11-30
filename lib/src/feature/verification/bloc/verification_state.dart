part of 'verification_bloc.dart';

abstract class VerificationState extends Equatable {
  bool get showStopConfirmation => true;

  bool get canGoBack => false;

  bool get didGoBack => false;

  double get stepperProgress => 0.0;

  Organization? get organization => null;

  const VerificationState();

  @override
  List<Object?> get props => [showStopConfirmation, canGoBack, didGoBack, stepperProgress, organization];
}

class VerificationInitial extends VerificationState {}

class VerificationLoadInProgress extends VerificationState {}

class VerificationGenericError extends VerificationState {
  @override
  bool get showStopConfirmation => false;
}

class VerificationCheckOrganization extends VerificationState {
  final VerificationFlow flow;
  final bool afterBackPressed;

  const VerificationCheckOrganization(this.flow, {this.afterBackPressed = false});

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.25;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  Organization? get organization => flow.organization;
}

class VerificationMissingAttributes extends VerificationState {
  final VerificationFlow flow;

  const VerificationMissingAttributes(this.flow);

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.5;

  @override
  bool get canGoBack => true;

  @override
  bool get showStopConfirmation => false;

  @override
  Organization? get organization => flow.organization;
}

class VerificationConfirmDataAttributes extends VerificationState {
  final VerificationFlow flow;
  final bool afterBackPressed;

  const VerificationConfirmDataAttributes(this.flow, {this.afterBackPressed = false});

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.5;

  @override
  bool get didGoBack => afterBackPressed;

  @override
  Organization? get organization => flow.organization;

  @override
  bool get canGoBack => true;
}

class VerificationConfirmPin extends VerificationState {
  final VerificationFlow flow;

  const VerificationConfirmPin(this.flow);

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 0.75;

  @override
  Organization? get organization => flow.organization;

  @override
  bool get canGoBack => true;
}

class VerificationSuccess extends VerificationState {
  final VerificationFlow flow;

  const VerificationSuccess(this.flow);

  @override
  List<Object?> get props => [flow, ...super.props];

  @override
  double get stepperProgress => 1;

  @override
  Organization? get organization => flow.organization;

  @override
  bool get showStopConfirmation => false;
}

class VerificationStopped extends VerificationState {
  const VerificationStopped();

  @override
  double get stepperProgress => 1;

  @override
  bool get showStopConfirmation => false;
}
