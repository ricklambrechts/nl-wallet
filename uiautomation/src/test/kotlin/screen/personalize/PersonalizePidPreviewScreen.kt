package screen.personalize

import util.MobileActions

class PersonalizePidPreviewScreen : MobileActions() {

    private val screen = find.byValueKey("personalizePidPreviewPage")

    private val genderText = find.byText("Vrouw")
    private val addressText = find.byText("Van Wijngaerdenstraat 1, 2596TW Toetsoog")

    private val acceptButton = find.byValueKey("acceptButton")
    private val rejectButton = find.byValueKey("rejectButton")

    fun visible() = isElementVisible(screen)

    fun humanReadablePidDataVisible() = isElementVisible(genderText) && isElementVisible(addressText)

    fun confirmButtonsVisible() = isElementVisible(acceptButton) && isElementVisible(rejectButton)

    fun clickAcceptButton() = clickElement(acceptButton)

    fun clickRejectButton() = clickElement(rejectButton)

    fun switchToApp() = switchToAppContext()
}
