package feature.card

import helper.TestBase
import navigator.CardNavigator
import navigator.screen.CardScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junitpioneer.jupiter.RetryingTest
import screen.card.CardDataScreen
import screen.card.CardDetailScreen
import screen.card.CardHistoryScreen
import screen.dashboard.DashboardScreen

@DisplayName("${CardDetailTests.USE_CASE} App shows card detail overview [${CardDetailTests.JIRA_ID}]")
class CardDetailTests : TestBase() {

    companion object {
        const val USE_CASE = "UC 7.2"
        const val JIRA_ID = "PVW-1228"
    }

    private lateinit var cardDetailScreen: CardDetailScreen

    @BeforeEach
    fun setUp() {
        CardNavigator().toScreen(CardScreen.CardDetail)

        cardDetailScreen = CardDetailScreen()
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.1 The Card detail page shows the actual card data as stored in the app. [${JIRA_ID}]")
    fun verifyCardDetailScreen() {
        assertTrue(cardDetailScreen.visible(), "card detail screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.2 The Card detail page shows the Card face (exactly the same as on the dashboard, minus the 'show details' button). [${JIRA_ID}]")
    fun verifyCardDetailButtonAbsent() {
        assertTrue(cardDetailScreen.cardFaceElements(), "card face for detail screen is not visible and/or correct")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.3 The Card detail page shows: issuer name, empty history state. [${JIRA_ID}]")
    fun verifyDataAndHistoryState() {
        assertTrue(cardDetailScreen.issuerAndHistoryStates(), "issuer and/or history state not not visible and/or correct")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.4 The Card detail page offers a button to reveal the card attributes. [${JIRA_ID}]")
    fun verifyCardDataButton() {
        cardDetailScreen.clickCardDataButton()

        val cardDataScreen = CardDataScreen()
        assertTrue(cardDataScreen.visible(), "card data screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.5 The Card detail page offers a button to display card history. [${JIRA_ID}]")
    fun verifyCardHistoryButton() {
        cardDetailScreen.clickCardHistoryButton()

        val cardHistoryScreen = CardHistoryScreen()
        assertTrue(cardHistoryScreen.visible(), "card history screen is not visible")
    }

    @RetryingTest(value = MAX_RETRY_COUNT, name = "{displayName} - {index}")
    @DisplayName("$USE_CASE.6 The Card detail page offers a button to go back to the card overview. [${JIRA_ID}]")
    fun verifyBackButton() {
        cardDetailScreen.clickBottomBackButton()

        val dashboardScreen = DashboardScreen()
        assertTrue(dashboardScreen.visible(), "dashboard screen is not visible")
    }
}
