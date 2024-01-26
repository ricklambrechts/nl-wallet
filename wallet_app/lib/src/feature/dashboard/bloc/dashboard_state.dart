part of 'dashboard_bloc.dart';

sealed class DashboardState extends Equatable {
  const DashboardState();
}

class DashboardStateInitial extends DashboardState {
  const DashboardStateInitial();

  @override
  List<Object> get props => [];
}

class DashboardLoadInProgress extends DashboardState {
  const DashboardLoadInProgress();

  @override
  List<Object> get props => [];
}

class DashboardLoadSuccess extends DashboardState {
  final List<WalletCard> cards;
  final List<TimelineAttribute>? history;

  const DashboardLoadSuccess({
    required this.cards,
    this.history,
  });

  @override
  List<Object?> get props => [cards, history];
}

class DashboardLoadFailure extends DashboardState {
  const DashboardLoadFailure();

  @override
  List<Object> get props => [];
}
