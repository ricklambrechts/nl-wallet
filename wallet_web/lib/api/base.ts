import type { ErrorType } from "@/models/state"
import type { AxiosError } from "axios"

export const REQUEST_TIMEOUT = 5000 // half of the currently configured expiration time of the ephemeral session ID included in the session URL

export const catch_axios_error: <T>(e: AxiosError) => T = (e) => {
  console.error(e)
  if (e.code === "ECONNABORTED") {
    throw "timeout" as ErrorType
  } else {
    throw "failed" as ErrorType
  }
}
