package feature.dashboard

import helper.TestBase
import navigator.OnboardingNavigator
import navigator.OnboardingScreen
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.DisplayName
import org.junit.jupiter.api.Tag
import org.junit.jupiter.api.Tags
import org.junitpioneer.jupiter.RetryingTest
import screen.card.CardDetailScreen
import screen.dashboard.DashboardScreen

@DisplayName("UC 7.1 - App shows all cards available in the app [PVW-1227]")
class DashboardTests : TestBase() {

    private lateinit var dashboardScreen: DashboardScreen

    @BeforeEach
    fun setUp() {
        OnboardingNavigator().toScreen(OnboardingScreen.Dashboard)

        dashboardScreen = DashboardScreen()
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("1. The card overview page displays all cards currently available in the app.")
    fun verifyIssuedCardsVisible() {
        assertTrue(dashboardScreen.cardsVisible(), "Expected cards are not visible")
    }

    /*@RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("2. Each card is recognizable as a physical card (fixed ratio, unless the font size is too big, then the card ratio constraint is relaxed) and includes the following: a title, subtitle, background image, logo, CTA button.")
    fun verifyCardPhysicalFixedRatioAndFaceElements() {
        // Manual test: https://SSSS/jira/browse/PVW-1976
    }*/

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("3. The card information (and images) is displayed in the active language.")
    @Tags(Tag("english"))
    fun verifyActiveLanguage() {
        assertTrue(dashboardScreen.cardFaceTextsInActiveLanguage(), "Card face texts are not in active language")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("4. Tapping the card opens the card's details [UC 7.2]")
    fun verifyCardDetailScreen() {
        dashboardScreen.clickPidCard()

        val cardDetailScreen = CardDetailScreen()
        assertTrue(cardDetailScreen.visible(), "card detail screen is not visible")
    }

    @RetryingTest(MAX_RETRY_COUNT)
    @DisplayName("5. The card sorting is initially fixed: PID is first, Address is second.")
    fun verifyCardsFixedSorting() {
        assertTrue(dashboardScreen.checkCardSorting(), "card sorting not as expected")
    }
}
