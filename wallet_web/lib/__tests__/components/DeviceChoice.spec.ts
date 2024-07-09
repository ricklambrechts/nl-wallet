import DeviceChoice from "@/components/DeviceChoice.vue"
import { type SessionType } from "@/models/status"
import { translations, translationsKey } from "@/util/translations"
import { mount } from "@vue/test-utils"
import { describe, expect, it, vi } from "vitest"

await import("../setup")

// vi.mock("@/api/session")
vi.mock("@/api/status")

await import("../setup")

describe("DeviceChoice", () => {
  it("should show same device link with UL", async () => {
    const wrapper = mount(DeviceChoice, {
      props: { ul: "ul_123" },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })
    const button = wrapper.find("[data-testid=same_device_button]")
    expect(button.exists()).toBe(true)
    expect(button.attributes("href")).toEqual("ul_123")
  })

  it("should emit choice", async () => {
    const wrapper = mount(DeviceChoice, {
      props: { ul: "ul_123" },
      global: { provide: { [translationsKey as symbol]: translations("nl") } },
    })

    await wrapper.find("[data-testid=cross_device_button]").trigger("click")
    expect(wrapper.emitted()).toHaveProperty("choice")
    expect(wrapper.emitted().choice[0]).toEqual(["cross_device"])
  })
})
